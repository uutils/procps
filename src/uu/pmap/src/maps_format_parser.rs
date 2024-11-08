// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt;
use std::io::{Error, ErrorKind};

// Represents a parsed single line from /proc/<PID>/maps for the default and device formats. It
// omits the inode information because it's not used by those formats.
#[derive(Debug, PartialEq)]
pub struct MapLine {
    pub address: String,
    pub size_in_kb: u64,
    pub perms: Perms,
    pub offset: String,
    pub device: String,
    pub mapping: String,
}

// Represents a set of permissions from the "perms" column of /proc/<PID>/maps.
#[derive(Clone, Copy, Debug, PartialEq)]
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

// Please note: While `Perms` has four boolean fields, it's string representation has five
// characters because pmap's default and device formats use five characters for the perms,
// with the last character always being '-'.
impl fmt::Display for Perms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
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

    // skip the "inode" column
    let mapping: String = rest.splitn(2, ' ').skip(1).collect();
    let mapping = mapping.trim_ascii_start();
    let mapping = parse_mapping(mapping);

    Ok(MapLine {
        address,
        size_in_kb,
        perms,
        offset,
        device,
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

fn parse_mapping(mapping: &str) -> String {
    if mapping == "[stack]" {
        return "  [ stack ]".into();
    }

    if mapping.is_empty() || mapping.starts_with('[') || mapping.starts_with("anon") {
        return "  [ anon ]".into();
    }

    match mapping.rsplit_once('/') {
        Some((_, name)) => name.into(),
        None => mapping.into(),
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
        mapping: &str,
    ) -> MapLine {
        MapLine {
            address: address.to_string(),
            size_in_kb,
            perms,
            offset: offset.to_string(),
            device: device.to_string(),
            mapping: mapping.to_string(),
        }
    }

    #[test]
    fn test_perms_to_string() {
        assert_eq!("-----", Perms::from("---p").to_string());
        assert_eq!("---s-", Perms::from("---s").to_string());
        assert_eq!("rwx--", Perms::from("rwxp").to_string());
    }

    #[test]
    fn test_parse_map_line() {
        let data = [
            (
                create_map_line("000062442eb9e000", 16, Perms::from("r--p"), "0000000000000000", "008:00008", "konsole"),
                "62442eb9e000-62442eba2000 r--p 00000000 08:08 10813151                   /usr/bin/konsole"
            ),
            (
                create_map_line("000071af50000000", 132, Perms::from("rw-p"), "0000000000000000", "000:00000", "  [ anon ]"),
                "71af50000000-71af50021000 rw-p 00000000 00:00 0 "
            ),
            (
                create_map_line("00007ffc3f8df000", 132, Perms::from("rw-p"), "0000000000000000", "000:00000", "  [ stack ]"),
                "7ffc3f8df000-7ffc3f900000 rw-p 00000000 00:00 0                          [stack]"
            ),
            (
                create_map_line("000071af8c9e6000", 16, Perms::from("rw-s"), "0000000105830000", "000:00010", "  [ anon ]"),
                "71af8c9e6000-71af8c9ea000 rw-s 105830000 00:10 1075                      anon_inode:i915.gem"
            ),
            (
                create_map_line("000071af6cf0c000", 3560, Perms::from("rw-s"), "0000000000000000", "000:00001", "memfd:wayland-shm (deleted)"),
                "71af6cf0c000-71af6d286000 rw-s 00000000 00:01 256481                     /memfd:wayland-shm (deleted)"
            ),
            (
                create_map_line("ffffffffff600000", 4, Perms::from("--xp"), "0000000000000000", "000:00000", "  [ anon ]"),
                "ffffffffff600000-ffffffffff601000 --xp 00000000 00:00 0                  [vsyscall]"
            ),
            (
                create_map_line("00005e8187da8000", 24, Perms::from("r--p"), "0000000000000000", "008:00008", "hello   world"),
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
        assert_eq!("  [ anon ]", parse_mapping(""));
        assert_eq!("  [ anon ]", parse_mapping("[vvar]"));
        assert_eq!("  [ anon ]", parse_mapping("[vdso]"));
        assert_eq!("  [ anon ]", parse_mapping("anon_inode:i915.gem"));
        assert_eq!("  [ stack ]", parse_mapping("[stack]"));
        assert_eq!(
            "ld-linux-x86-64.so.2",
            parse_mapping("/usr/lib/ld-linux-x86-64.so.2")
        );
    }
}
