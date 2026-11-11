//! Reads the TV information sheet from the system, once each time the screen opens.
//!
//! Everything comes from Rust (system properties, `/proc`, `statvfs`, `getifaddrs` and the GL
//! driver) except the display mode, which only the Java bridge can tell.

use lume_core::device_info::{self, DeviceInfo, Field, Link, Value};
use std::ffi::{CStr, CString};
use std::net::Ipv4Addr;

/// A system property, `None` when it is unset or empty.
fn property(name: &str) -> Option<String> {
    let name = CString::new(name).ok()?;
    let mut buffer = [0 as libc::c_char; 92];
    // SAFETY: `buffer` holds the 92 bytes (`PROP_VALUE_MAX`) the call may write, NUL included.
    let length = unsafe { libc::__system_property_get(name.as_ptr(), buffer.as_mut_ptr()) };
    if length <= 0 {
        return None;
    }
    // SAFETY: the call wrote a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_string_lossy();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The first of these properties that is set.
fn first_property(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| property(name))
}

/// The kernel release (`5.4.242-g1a2b`). `/proc/version` is closed to apps since Android 10.
fn kernel_release() -> Option<String> {
    // SAFETY: `uname` fills the zeroed struct or fails.
    let name = unsafe {
        let mut name: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut name) != 0 {
            return None;
        }
        name
    };
    // SAFETY: `release` is a NUL-terminated string after a successful call.
    let release = unsafe { CStr::from_ptr(name.release.as_ptr()) };
    Some(release.to_string_lossy().into_owned())
}

/// The bytes of the data partition: total and free.
fn storage() -> Option<(u64, u64)> {
    let path = CString::new("/data").ok()?;
    // SAFETY: `statvfs` fills the zeroed struct or fails.
    let stat = unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        stat
    };
    let unit = stat.f_frsize as u64;
    Some((stat.f_blocks as u64 * unit, stat.f_bavail as u64 * unit))
}

/// How the TV reaches the network and its IPv4 address, from the interfaces that have one.
///
/// `getifaddrs` is blocked for apps on Android 11, but listing the IPv4 interfaces with
/// `SIOCGIFCONF` is not.
fn network() -> Option<(Link, Ipv4Addr)> {
    // SAFETY: a plain datagram socket, closed below.
    let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket < 0 {
        return None;
    }
    // SAFETY: an all-zero `ifreq` is a valid value.
    let mut requests: [libc::ifreq; 16] = unsafe { std::mem::zeroed() };
    let mut config = libc::ifconf {
        ifc_len: std::mem::size_of_val(&requests) as libc::c_int,
        ifc_ifcu: libc::__c_anonymous_ifc_ifcu {
            ifcu_req: requests.as_mut_ptr(),
        },
    };
    // SAFETY: `config` points at `requests`, which holds `ifc_len` bytes.
    let result = unsafe { libc::ioctl(socket, libc::SIOCGIFCONF as _, &mut config) };
    // SAFETY: `socket` is the descriptor opened above.
    unsafe { libc::close(socket) };
    if result != 0 {
        return None;
    }
    let count = config.ifc_len as usize / std::mem::size_of::<libc::ifreq>();
    // Wired beats wireless beats anything else (a tunnel, a mobile link).
    let rank = |link: Link| match link {
        Link::Ethernet => 0,
        Link::WiFi => 1,
        Link::Other => 2,
    };
    let mut best: Option<(Link, Ipv4Addr)> = None;
    for request in &requests[..count.min(requests.len())] {
        // SAFETY: the name is NUL-terminated (the array was zeroed and the kernel fills it).
        let name = unsafe { CStr::from_ptr(request.ifr_name.as_ptr()) }.to_string_lossy();
        // SAFETY: SIOCGIFCONF stores a `sockaddr_in` in the address of each request.
        let raw = unsafe {
            (&request.ifr_ifru.ifru_addr as *const libc::sockaddr)
                .cast::<libc::sockaddr_in>()
                .read_unaligned()
                .sin_addr
                .s_addr
        };
        let address = Ipv4Addr::from(u32::from_be(raw));
        if name == "lo" || address.is_unspecified() || address.is_loopback() {
            continue;
        }
        let link = if name.starts_with("wlan") {
            Link::WiFi
        } else if name.starts_with("eth") {
            Link::Ethernet
        } else {
            Link::Other
        };
        if best.is_none_or(|(b, _)| rank(link) < rank(b)) {
            best = Some((link, address));
        }
    }
    best
}

/// Reads the whole sheet. `gpu` is the driver's renderer and version strings; `display` is the
/// bridge's answer (resolution, refresh rate, density).
pub fn read(gpu: Option<(String, String)>, display: Option<[Value; 3]>) -> DeviceInfo {
    let mut info = DeviceInfo::default();
    let text = |field, value: Option<String>, info: &mut DeviceInfo| {
        if let Some(value) = value {
            info.set_text(field, &value);
        }
    };

    text(Field::Brand, property("ro.product.brand"), &mut info);
    text(Field::Model, property("ro.product.model"), &mut info);
    text(
        Field::Manufacturer,
        property("ro.product.manufacturer"),
        &mut info,
    );
    text(
        Field::Board,
        first_property(&["ro.product.board", "ro.product.device"]),
        &mut info,
    );

    text(
        Field::Soc,
        first_property(&["ro.soc.model", "ro.board.platform", "ro.hardware"]),
        &mut info,
    );
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        let (name, cores) = device_info::parse_cpuinfo(&cpuinfo);
        text(Field::Cpu, name, &mut info);
        if cores > 0 {
            info.set(Field::Cores, Value::Count(cores));
        }
    }
    text(
        Field::Abi,
        first_property(&["ro.product.cpu.abi", "ro.product.cpu.abilist"]),
        &mut info,
    );
    if let Some((renderer, version)) = gpu {
        text(Field::Gpu, Some(renderer), &mut info);
        // "OpenGL ES 3.2 v1.r32p1-01eac0": the first three words are the version.
        let short: Vec<&str> = version.split_whitespace().take(3).collect();
        text(Field::OpenGl, Some(short.join(" ")), &mut info);
    }

    if let Some(total) = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|t| device_info::parse_memory_total(&t))
    {
        info.set(Field::Ram, Value::Bytes(total));
    }
    let low_ram = property("ro.config.low_ram").is_some_and(|v| v == "true");
    info.set(Field::MemoryProfile, Value::LowRam(low_ram));
    if let Some((total, free)) = storage() {
        info.set(Field::StorageTotal, Value::Bytes(total));
        info.set(Field::StorageFree, Value::Bytes(free));
    }

    text(
        Field::Android,
        property("ro.build.version.release"),
        &mut info,
    );
    text(Field::Api, property("ro.build.version.sdk"), &mut info);
    text(Field::Firmware, property("ro.build.display.id"), &mut info);
    text(
        Field::SecurityPatch,
        property("ro.build.version.security_patch"),
        &mut info,
    );
    text(Field::Kernel, kernel_release(), &mut info);

    if let Some([size, hertz, dpi]) = display {
        info.set(Field::Resolution, size);
        info.set(Field::RefreshRate, hertz);
        info.set(Field::Density, dpi);
    }

    if let Some((link, address)) = network() {
        info.set(Field::Connection, Value::Link(link));
        info.set_text(Field::Ip, &address.to_string());
    }
    info
}
