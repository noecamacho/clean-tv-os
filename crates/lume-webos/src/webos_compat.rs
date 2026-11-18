//! The webOS NDK's libc lacks `getauxval`, which Rust's startup needs: read the real kernel
//! auxiliary values from `/proc/self/auxv` instead of inventing CPU capabilities.
//!
//! Adapted from Luma's `src/webos-compat.rs` (ecklf/lg-luma, MIT).

fn read_auxv(fd: libc::c_int, kind: libc::c_ulong) -> Option<libc::c_ulong> {
    loop {
        let mut entry = [0 as libc::c_ulong; 2];
        // SAFETY: the byte slice covers exactly this initialized, writable entry.
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                entry.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(&entry),
            )
        };
        let mut offset = 0;
        while offset < bytes.len() {
            // SAFETY: read receives a valid writable slice and never owns fd.
            let count = unsafe {
                libc::read(
                    fd,
                    bytes[offset..].as_mut_ptr().cast(),
                    bytes.len() - offset,
                )
            };
            if count < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return None;
            }
            if count == 0 {
                return None;
            }
            offset += count as usize;
        }
        if entry[0] == 0 {
            return None;
        }
        if entry[0] == kind {
            return Some(entry[1]);
        }
    }
}

#[no_mangle]
extern "C" fn getauxval(kind: libc::c_ulong) -> libc::c_ulong {
    // SAFETY: errno is thread-local; open uses a static NUL-terminated pathname.
    // This path needs no allocations, SDL, or initialized Rust application state.
    unsafe {
        let errno = libc::__errno_location();
        let saved_errno = *errno;
        let fd = libc::open(
            c"/proc/self/auxv".as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC,
        );
        let value = if fd >= 0 {
            let value = read_auxv(fd, kind);
            libc::close(fd);
            value
        } else {
            None
        };
        *errno = if value.is_some() {
            saved_errno
        } else {
            libc::ENOENT
        };
        value.unwrap_or(0)
    }
}
