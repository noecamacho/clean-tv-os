//! The TV information screen: what the device is, as the platform reads it once on entering.
//!
//! The platform fills a [`DeviceInfo`] field by field; the screen draws [`SECTIONS`] in order and
//! shows a dash for what the system would not tell. The text parsers here take the contents of
//! `/proc` files and the Java bridge's `display` answer, so they are tested off the device.

use crate::i18n::{StringId, Translator};
use crate::stats::format_size;

/// One line of the information sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Brand,
    Model,
    Manufacturer,
    Board,
    Soc,
    Cpu,
    Cores,
    Abi,
    Gpu,
    OpenGl,
    Ram,
    MemoryProfile,
    StorageTotal,
    StorageFree,
    Android,
    Api,
    Firmware,
    SecurityPatch,
    Kernel,
    Resolution,
    RefreshRate,
    Density,
    Connection,
    Ip,
}

/// How the TV is connected to the network.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    WiFi,
    Ethernet,
    Other,
}

/// What a line holds. Numbers and kinds are formatted for the language when drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Bytes(u64),
    Count(u32),
    LowRam(bool),
    Link(Link),
    Hertz(f32),
    Dpi(u32),
}

/// A titled group of lines.
pub struct Section {
    pub title: StringId,
    pub lines: &'static [(Field, StringId)],
}

pub const SECTIONS: [Section; 6] = [
    Section {
        title: StringId::TvSectionDevice,
        lines: &[
            (Field::Brand, StringId::TvBrand),
            (Field::Model, StringId::TvModel),
            (Field::Manufacturer, StringId::TvManufacturer),
            (Field::Board, StringId::TvBoard),
        ],
    },
    Section {
        title: StringId::TvSectionProcessing,
        lines: &[
            (Field::Soc, StringId::TvSoc),
            (Field::Cpu, StringId::TvCpu),
            (Field::Cores, StringId::TvCores),
            (Field::Abi, StringId::TvAbi),
            (Field::Gpu, StringId::TvGpu),
            (Field::OpenGl, StringId::TvOpengl),
        ],
    },
    Section {
        title: StringId::TvSectionMemory,
        lines: &[
            (Field::Ram, StringId::TvRam),
            (Field::MemoryProfile, StringId::TvMemoryProfile),
            (Field::StorageTotal, StringId::TvStorageTotal),
            (Field::StorageFree, StringId::TvStorageFree),
        ],
    },
    Section {
        title: StringId::TvSectionSystem,
        lines: &[
            (Field::Android, StringId::TvAndroid),
            (Field::Api, StringId::TvApi),
            (Field::Firmware, StringId::TvFirmware),
            (Field::SecurityPatch, StringId::TvPatch),
            (Field::Kernel, StringId::TvKernel),
        ],
    },
    Section {
        title: StringId::TvSectionDisplay,
        lines: &[
            (Field::Resolution, StringId::TvResolution),
            (Field::RefreshRate, StringId::TvRefresh),
            (Field::Density, StringId::TvDensity),
        ],
    },
    Section {
        title: StringId::TvSectionNetwork,
        lines: &[
            (Field::Connection, StringId::TvConnection),
            (Field::Ip, StringId::TvIp),
        ],
    },
];

/// What the platform could read. A field it could not read is simply absent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceInfo {
    values: Vec<(Field, Value)>,
}

impl DeviceInfo {
    pub fn set(&mut self, field: Field, value: Value) {
        self.values.retain(|(f, _)| *f != field);
        self.values.push((field, value));
    }

    /// Sets a text field, unless the text is empty.
    pub fn set_text(&mut self, field: Field, text: &str) {
        let text = text.trim();
        if !text.is_empty() {
            self.set(field, Value::Text(text.to_string()));
        }
    }

    pub fn get(&self, field: Field) -> Option<&Value> {
        self.values
            .iter()
            .find(|(f, _)| *f == field)
            .map(|(_, v)| v)
    }

    /// The line as the screen shows it: a dash when it is missing.
    pub fn display(&self, field: Field, t: &Translator) -> String {
        let comma = t.language().starts_with("pt");
        match self.get(field) {
            None => "—".to_string(),
            Some(Value::Text(text)) => text.clone(),
            Some(Value::Bytes(bytes)) => format_size(*bytes, comma),
            Some(Value::Count(n)) => n.to_string(),
            Some(Value::LowRam(low)) => t
                .get(if *low {
                    StringId::TvProfileLow
                } else {
                    StringId::TvProfileStandard
                })
                .into_owned(),
            Some(Value::Link(link)) => t
                .get(match link {
                    Link::WiFi => StringId::TvLinkWifi,
                    Link::Ethernet => StringId::TvLinkEthernet,
                    Link::Other => StringId::TvLinkOther,
                })
                .into_owned(),
            Some(Value::Hertz(hz)) => {
                let text = if (hz - hz.round()).abs() < 0.05 {
                    format!("{hz:.0} Hz")
                } else {
                    format!("{hz:.1} Hz")
                };
                if comma {
                    text.replace('.', ",")
                } else {
                    text
                }
            }
            Some(Value::Dpi(dpi)) => format!("{dpi} dpi"),
        }
    }
}

/// The interface size, refresh rate and density from the bridge's `display` answer:
/// `width TAB height TAB hertz TAB dpi`. `None` when it is malformed.
pub fn parse_display(text: &str) -> Option<[Value; 3]> {
    let mut fields = text.trim().split('\t');
    let width: u32 = fields.next()?.parse().ok()?;
    let height: u32 = fields.next()?.parse().ok()?;
    let hertz: f32 = fields.next()?.parse().ok()?;
    let dpi: u32 = fields.next()?.parse().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some([
        Value::Text(format!("{width} × {height}")),
        if hertz > 0.0 {
            Value::Hertz(hertz)
        } else {
            Value::Text("—".to_string())
        },
        Value::Dpi(dpi),
    ])
}

/// The CPU name and the number of cores from the contents of `/proc/cpuinfo`.
///
/// On ARM the name comes from the part number of the first core (`Cortex-A55`); where that is
/// unknown, from the `Hardware` or `model name` line.
pub fn parse_cpuinfo(text: &str) -> (Option<String>, u32) {
    let mut cores = 0;
    let (mut implementer, mut part) = (None, None);
    let (mut hardware, mut model) = (None, None);
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "processor" => cores += 1,
            "CPU implementer" if implementer.is_none() => implementer = Some(value),
            "CPU part" if part.is_none() => part = Some(value),
            "Hardware" if hardware.is_none() => hardware = Some(value),
            "model name" if model.is_none() => model = Some(value),
            _ => {}
        }
    }
    let cortex = match (implementer, part) {
        (Some("0x41"), Some(part)) => arm_core_name(part),
        _ => None,
    };
    let name = cortex
        .map(str::to_string)
        .or_else(|| hardware.filter(|v| !v.is_empty()).map(str::to_string))
        .or_else(|| model.filter(|v| !v.is_empty()).map(str::to_string));
    (name, cores)
}

/// Arm's own cores by part number.
fn arm_core_name(part: &str) -> Option<&'static str> {
    Some(match part {
        "0xc07" => "Cortex-A7",
        "0xc09" => "Cortex-A9",
        "0xc0f" => "Cortex-A15",
        "0xd03" => "Cortex-A53",
        "0xd04" => "Cortex-A35",
        "0xd05" => "Cortex-A55",
        "0xd07" => "Cortex-A57",
        "0xd08" => "Cortex-A72",
        "0xd09" => "Cortex-A73",
        "0xd0a" => "Cortex-A75",
        "0xd0b" => "Cortex-A76",
        "0xd0d" => "Cortex-A77",
        "0xd41" => "Cortex-A78",
        "0xd46" => "Cortex-A510",
        _ => return None,
    })
}

/// The total memory, in bytes, from the contents of `/proc/meminfo`.
pub fn parse_memory_total(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_display_answer_is_read() {
        let [size, hertz, dpi] = parse_display("1920\t1080\t60.0\t213\n").unwrap();
        assert_eq!(size, Value::Text("1920 × 1080".into()));
        assert_eq!(hertz, Value::Hertz(60.0));
        assert_eq!(dpi, Value::Dpi(213));
        assert!(parse_display("1920\t1080").is_none());
        assert!(parse_display("0\t0\t60\t213").is_none());
    }

    #[test]
    fn an_arm_core_is_named_by_its_part() {
        let text = "processor\t: 0\nCPU implementer\t: 0x41\nCPU part\t: 0xd05\n\
                    processor\t: 1\nCPU implementer\t: 0x41\nCPU part\t: 0xd05\nHardware\t: Amlogic\n";
        assert_eq!(parse_cpuinfo(text), (Some("Cortex-A55".to_string()), 2));
    }

    #[test]
    fn the_cpu_falls_back_to_the_hardware_or_model_line() {
        let text = "processor : 0\nmodel name : ARMv7 Processor rev 4 (v7l)\nHardware : Realtek\n";
        assert_eq!(parse_cpuinfo(text), (Some("Realtek".to_string()), 1));
        let text = "processor : 0\nmodel name : Intel(R) Atom\nprocessor : 1\n";
        assert_eq!(parse_cpuinfo(text), (Some("Intel(R) Atom".to_string()), 2));
        assert_eq!(parse_cpuinfo(""), (None, 0));
    }

    #[test]
    fn the_memory_total_is_in_bytes() {
        let text = "MemTotal:        1862404 kB\nMemFree: 1 kB\n";
        assert_eq!(parse_memory_total(text), Some(1_862_404 * 1024));
        assert_eq!(parse_memory_total("MemFree: 1 kB"), None);
    }

    #[test]
    fn a_missing_line_is_a_dash_and_values_follow_the_language() {
        let en = Translator::new(&["en"]);
        let pt = Translator::new(&["pt-BR"]);
        let mut info = DeviceInfo::default();
        assert_eq!(info.display(Field::Model, &en), "—");
        info.set_text(Field::Model, "  55C715 ");
        info.set_text(Field::Brand, "   ");
        info.set(Field::Ram, Value::Bytes(1_610_612_736));
        info.set(Field::RefreshRate, Value::Hertz(59.94));
        info.set(Field::Connection, Value::Link(Link::WiFi));
        assert_eq!(info.display(Field::Model, &en), "55C715");
        assert_eq!(info.display(Field::Brand, &en), "—");
        assert_eq!(info.display(Field::Ram, &en), "1.5 GB");
        assert_eq!(info.display(Field::Ram, &pt), "1,5 GB");
        assert_eq!(info.display(Field::RefreshRate, &pt), "59,9 Hz");
        assert_eq!(info.display(Field::Connection, &pt), "Wi-Fi");
    }

    #[test]
    fn every_field_is_on_the_sheet_once() {
        let count: usize = SECTIONS.iter().map(|s| s.lines.len()).sum();
        assert_eq!(count, 24);
    }
}
