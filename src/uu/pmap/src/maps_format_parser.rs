// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::pmap_config::PmapConfig;
use std::fmt;
use std::io::{Error, ErrorKind};

// Represents a parsed single line from /proc/<PID>/maps.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapLine {
    pub address: String,
    pub size_in_kb: u64,
    pub perms: Perms,
    pub offset: String,
    pub device: String,
    pub inode: u64,
    pub mapping: String,
}

// Represents a set of permissions from the "perms" column of /proc/<PID>/maps.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Perms {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub shared: bool,
}

impl From<&str> for Perms {
    fn from(s: &str) -> Self {
        let mut chars = s.chars();

        Self {
            readable: chars.next() == Some('r'),
            writable: chars.next() == Some('w'),
            executable: chars.next() == Some('x'),
            shared: chars.next() == Some('s'),
        }
    }
}

impl fmt::Display for Perms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.readable { 'r' } else { '-' },
            if self.writable { 'w' } else { '-' },
            if self.executable { 'x' } else { '-' },
            if self.shared { 's' } else { 'p' },
        )
    }
}

// Please note: While `Perms` has four boolean fields, its `Mode` representation
// used in pmap's default and device formats has five characters for the perms,
// with the last character always being '-'.
impl Perms {
    pub fn mode(&self) -> String {
        format!(
            "{}{}{}{}-",
            if self.readable { 'r' } else { '-' },
            if self.writable { 'w' } else { '-' },
            if self.executable { 'x' } else { '-' },
            if self.shared { 's' } else { '-' },
        )
    }
}

// Parses a single line from /proc/<PID>/maps. See
// https://www.kernel.org/doc/html/latest/filesystems/proc.html for details about the expected
// format.
//
// # Errors
//
// Will return an `Error` if the format is incorrect.
pub fn parse_map_line(line: &str) -> Result<MapLine, Error> {
    let (memory_range, rest) = line
        .split_once(' ')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    let (address, size_in_kb) = parse_address(memory_range)?;

    let (perms, rest) = rest
        .split_once(' ')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    let perms = Perms::from(perms);

    let (offset, rest) = rest
        .split_once(' ')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    let offset = format!("{offset:0>16}");

    let (device, rest) = rest
        .split_once(' ')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    let device = parse_device(device)?;

    let (inode, mapping) = rest
        .split_once(' ')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    let inode = inode
        .parse::<u64>()
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let mapping = mapping.trim_ascii_start().to_string();

    Ok(MapLine {
        address,
        size_in_kb,
        perms,
        offset,
        device,
        inode,
        mapping,
    })
}

// Returns the start address and the size of the provided memory range. The start address is always
// 16-digits and padded with 0, if necessary. The size is in KB.
fn parse_address(memory_range: &str) -> Result<(String, u64), Error> {
    let (start, end) = memory_range
        .split_once('-')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;

    let low = u64::from_str_radix(start, 16).map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let high = u64::from_str_radix(end, 16).map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let size_in_kb = (high - low) / 1024;

    Ok((format!("{start:0>16}"), size_in_kb))
}

// Pads the device info from /proc/<PID>/maps with zeros and turns AB:CD into 0AB:000CD.
fn parse_device(device: &str) -> Result<String, Error> {
    let (major, minor) = device
        .split_once(':')
        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
    Ok(format!("{major:0>3}:{minor:0>5}"))
}

impl MapLine {
    pub fn parse_mapping(&self, pmap_config: &PmapConfig) -> String {
        if pmap_config.custom_format_enabled {
            if self.mapping.starts_with('[') {
                return self.mapping.clone();
            }
        } else {
            if self.mapping == "[stack]" {
                return "  [ stack ]".into();
            }

            if self.mapping.is_empty()
                || self.mapping.starts_with('[')
                || self.mapping.starts_with("anon")
            {
                return "  [ anon ]".into();
            }
        }

        if pmap_config.show_path {
            self.mapping.clone()
        } else {
            match self.mapping.rsplit_once('/') {
                Some((_, name)) => name.into(),
                None => self.mapping.clone(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_map_line(
        address: &str,
        size_in_kb: u64,
        perms: Perms,
        offset: &str,
        device: &str,
        inode: u64,
        mapping: &str,
    ) -> MapLine {
        MapLine {
            address: address.to_string(),
            size_in_kb,
            perms,
            offset: offset.to_string(),
            device: device.to_string(),
            inode,
            mapping: mapping.to_string(),
        }
    }

    #[test]
    fn test_perms_to_string() {
        assert_eq!("---p", Perms::from("---p").to_string());
        assert_eq!("---s", Perms::from("---s").to_string());
        assert_eq!("rwxp", Perms::from("rwxp").to_string());
    }

    #[test]
    fn test_perms_mode() {
        assert_eq!("-----", Perms::from("---p").mode());
        assert_eq!("---s-", Perms::from("---s").mode());
        assert_eq!("rwx--", Perms::from("rwxp").mode());
    }

    #[test]
    fn test_parse_map_line() {
        let data = [
            (
                create_map_line("000062442eb9e000", 16, Perms::from("r--p"), "0000000000000000", "008:00008", 10813151, "/usr/bin/konsole"),
                "62442eb9e000-62442eba2000 r--p 00000000 08:08 10813151                   /usr/bin/konsole"
            ),
            (
                create_map_line("000071af50000000", 132, Perms::from("rw-p"), "0000000000000000", "000:00000", 0, ""),
                "71af50000000-71af50021000 rw-p 00000000 00:00 0 "
            ),
            (
                create_map_line("00007ffc3f8df000", 132, Perms::from("rw-p"), "0000000000000000", "000:00000", 0, "[stack]"),
                "7ffc3f8df000-7ffc3f900000 rw-p 00000000 00:00 0                          [stack]"
            ),
            (
                create_map_line("000071af8c9e6000", 16, Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "anon_inode:i915.gem"),
                "71af8c9e6000-71af8c9ea000 rw-s 105830000 00:10 1075                      anon_inode:i915.gem"
            ),
            (
                create_map_line("000071af6cf0c000", 3560, Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "/memfd:wayland-shm (deleted)"),
                "71af6cf0c000-71af6d286000 rw-s 00000000 00:01 256481                     /memfd:wayland-shm (deleted)"
            ),
            (
                create_map_line("ffffffffff600000", 4, Perms::from("--xp"), "0000000000000000", "000:00000", 0, "[vsyscall]"),
                "ffffffffff600000-ffffffffff601000 --xp 00000000 00:00 0                  [vsyscall]"
            ),
            (
                create_map_line("00005e8187da8000", 24, Perms::from("r--p"), "0000000000000000", "008:00008", 9524160, "/usr/bin/hello   world"),
                "5e8187da8000-5e8187dae000 r--p 00000000 08:08 9524160                    /usr/bin/hello   world"
            ),
        ];

        for (expected_map_line, line) in data {
            assert_eq!(expected_map_line, parse_map_line(line).unwrap());
        }
    }

    #[test]
    fn test_parse_map_line_with_invalid_format() {
        assert!(parse_map_line("invalid_format").is_err());
    }

    #[test]
    fn test_parse_address() {
        let (start, size) = parse_address("ffffffffff600000-ffffffffff601000").unwrap();
        assert_eq!(start, "ffffffffff600000");
        assert_eq!(size, 4);

        let (start, size) = parse_address("7ffc4f0c2000-7ffc4f0e3000").unwrap();
        assert_eq!(start, "00007ffc4f0c2000");
        assert_eq!(size, 132);
    }

    #[test]
    fn test_parse_address_with_missing_hyphen() {
        assert!(parse_address("ffffffffff600000").is_err());
    }

    #[test]
    fn test_parse_address_with_non_hex_values() {
        assert!(parse_address("zfffffffff600000-ffffffffff601000").is_err());
        assert!(parse_address("ffffffffff600000-zfffffffff601000").is_err());
    }

    #[test]
    fn test_parse_device() {
        assert_eq!("012:00034", parse_device("12:34").unwrap());
        assert_eq!("000:00000", parse_device("00:00").unwrap());
    }

    #[test]
    fn test_parse_device_without_colon() {
        assert!(parse_device("1234").is_err());
    }

    #[test]
    fn test_parse_mapping() {
        let mut mapline = MapLine::default();
        let mut pmap_config = PmapConfig::default();

        mapline.mapping = "".to_string();
        pmap_config.custom_format_enabled = false;
        pmap_config.show_path = false;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.custom_format_enabled = true;
        pmap_config.show_path = false;
        assert_eq!("", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("", mapline.parse_mapping(&pmap_config));

        mapline.mapping = "[vvar]".to_string();
        pmap_config.custom_format_enabled = false;
        pmap_config.show_path = false;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.custom_format_enabled = true;
        pmap_config.show_path = false;
        assert_eq!("[vvar]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("[vvar]", mapline.parse_mapping(&pmap_config));

        mapline.mapping = "anon_inode:i915.gem".to_string();
        pmap_config.custom_format_enabled = false;
        pmap_config.show_path = false;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("  [ anon ]", mapline.parse_mapping(&pmap_config));
        pmap_config.custom_format_enabled = true;
        pmap_config.show_path = false;
        assert_eq!("anon_inode:i915.gem", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("anon_inode:i915.gem", mapline.parse_mapping(&pmap_config));

        mapline.mapping = "[stack]".to_string();
        pmap_config.custom_format_enabled = false;
        pmap_config.show_path = false;
        assert_eq!("  [ stack ]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("  [ stack ]", mapline.parse_mapping(&pmap_config));
        pmap_config.custom_format_enabled = true;
        pmap_config.show_path = false;
        assert_eq!("[stack]", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!("[stack]", mapline.parse_mapping(&pmap_config));

        mapline.mapping = "/usr/lib/ld-linux-x86-64.so.2".to_string();
        pmap_config.custom_format_enabled = false;
        pmap_config.show_path = false;
        assert_eq!("ld-linux-x86-64.so.2", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!(
            "/usr/lib/ld-linux-x86-64.so.2",
            mapline.parse_mapping(&pmap_config)
        );
        pmap_config.custom_format_enabled = true;
        pmap_config.show_path = false;
        assert_eq!("ld-linux-x86-64.so.2", mapline.parse_mapping(&pmap_config));
        pmap_config.show_path = true;
        assert_eq!(
            "/usr/lib/ld-linux-x86-64.so.2",
            mapline.parse_mapping(&pmap_config)
        );
    }
}
