// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, Command};
use std::env;
use std::fs;
use std::io::Error;
use std::process;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pmap.md");
const USAGE: &str = help_usage!("pmap.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let pid = matches.get_one::<String>("pid").expect("PID required");

    match parse_cmdline(pid) {
        Ok(cmdline) => {
            println!("{}:   {}", pid, cmdline);
        }
        Err(_) => {
            process::exit(42);
        }
    }

    match parse_maps(pid) {
        // TODO calculate total
        Ok(_) => println!(" total"),
        Err(_) => {
            process::exit(1);
        }
    }

    Ok(())
}

fn parse_cmdline(pid: &str) -> Result<String, Error> {
    let path = format!("/proc/{}/cmdline", pid);
    let contents = fs::read(path)?;
    // Command line arguments are separated by null bytes.
    // Replace them with spaces for display.
    let cmdline = contents
        .split(|&c| c == 0)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect::<Vec<&str>>()
        .join(" ");
    Ok(cmdline)
}

fn parse_maps(pid: &str) -> Result<(), Error> {
    let path = format!("/proc/{}/maps", pid);
    let contents = fs::read_to_string(path)?;

    for line in contents.lines() {
        println!("{}", parse_map_line(line));
    }

    Ok(())
}

// Parses a single line from /proc/<PID>/maps.
fn parse_map_line(line: &str) -> String {
    let (memory_range, rest) = line.split_once(' ').expect("line should contain ' '");
    let (start_address, size_in_kb) = parse_memory_range(memory_range);

    let (perms, rest) = rest.split_once(' ').expect("line should contain 2nd ' '");
    let perms = parse_perms(perms);

    let filename: String = rest.splitn(4, " ").skip(3).collect();
    let filename = filename.trim_ascii_start();
    let filename = parse_filename(filename);

    format!("{start_address} {size_in_kb:>6}K {perms} {filename}")
}

// Returns the start address and the size of the provided memory range. The start address is always
// 16-digits and padded with 0, if necessary. The size is in KB.
//
// This function assumes the provided `memory_range` comes from /proc/<PID>/maps and thus its
// format is correct.
fn parse_memory_range(memory_range: &str) -> (String, u64) {
    let (start, end) = memory_range
        .split_once('-')
        .expect("memory range should contain '-'");

    let low = u64::from_str_radix(start, 16).expect("should be a hex value");
    let high = u64::from_str_radix(end, 16).expect("should be a hex value");
    let size_in_kb = (high - low) / 1024;

    (format!("{start:0>16}"), size_in_kb)
}

// Turns a 4-char perms string from /proc/<PID>/maps into a 5-char perms string. The first three
// chars are left untouched.
fn parse_perms(perms: &str) -> String {
    let perms = perms.replace("p", "-");

    // the fifth char seems to be always '-' in the original pmap
    format!("{perms}-")
}

fn parse_filename(filename: &str) -> String {
    if filename == "[stack]" {
        return "  [ stack ]".into();
    }

    if filename.is_empty() || filename.starts_with('[') || filename.starts_with("anon") {
        return "  [ anon ]".into();
    }

    match filename.rsplit_once('/') {
        Some((_, name)) => name.into(),
        None => filename.into(),
    }
}

pub fn uu_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new("pid")
                .help("Process ID")
                .required_unless_present_any(["create-rc", "create-rc-to"]) // Adjusted for -n, -N note
                .action(ArgAction::Set)
                .conflicts_with_all(["create-rc", "create-rc-to"]),
        ) // Ensure pid is not used with -n, -N
        .arg(
            Arg::new("extended")
                .short('x')
                .long("extended")
                .help("show details"),
        )
        .arg(
            Arg::new("very-extended")
                .short('X')
                .help("show even more details"),
        )
        .arg(
            Arg::new("all-details")
                .long("XX")
                .help("show everything the kernel provides"),
        )
        .arg(
            Arg::new("read-rc")
                .short('c')
                .long("read-rc")
                .help("read the default rc"),
        )
        .arg(
            Arg::new("read-rc-from")
                .short('C')
                .long("read-rc-from")
                .num_args(1)
                .help("read the rc from file"),
        )
        .arg(
            Arg::new("create-rc")
                .short('n')
                .long("create-rc")
                .help("create new default rc"),
        )
        .arg(
            Arg::new("create-rc-to")
                .short('N')
                .long("create-rc-to")
                .num_args(1)
                .help("create new rc to file"),
        )
        .arg(
            Arg::new("device")
                .short('d')
                .long("device")
                .help("show the device format"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("do not display header and footer"),
        )
        .arg(
            Arg::new("show-path")
                .short('p')
                .long("show-path")
                .help("show path in the mapping"),
        )
        .arg(
            Arg::new("range")
                .short('A')
                .long("range")
                .num_args(1..=2)
                .help("limit results to the given range"),
        )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_map_line() {
        let data = [
            (
                "000062442eb9e000     16K r---- konsole",
                "62442eb9e000-62442eba2000 r--p 00000000 08:08 10813151                   /usr/bin/konsole"
            ),
            (
                "000071af50000000    132K rw---   [ anon ]",
                "71af50000000-71af50021000 rw-p 00000000 00:00 0 "
            ),
            (
                "00007ffc3f8df000    132K rw---   [ stack ]",
                "7ffc3f8df000-7ffc3f900000 rw-p 00000000 00:00 0                          [stack]"
            ),
            (
                "000071af8c9e6000     16K rw-s-   [ anon ]",
                "71af8c9e6000-71af8c9ea000 rw-s 105830000 00:10 1075                      anon_inode:i915.gem"
            ),
            (
                "000071af6cf0c000   3560K rw-s- memfd:wayland-shm (deleted)",
                "71af6cf0c000-71af6d286000 rw-s 00000000 00:01 256481                     /memfd:wayland-shm (deleted)"
            ),
            (
                "ffffffffff600000      4K --x--   [ anon ]",
                "ffffffffff600000-ffffffffff601000 --xp 00000000 00:00 0                  [vsyscall]"
            ),
            (
                "00005e8187da8000     24K r---- hello   world",
                "5e8187da8000-5e8187dae000 r--p 00000000 08:08 9524160                    /usr/bin/hello   world"
            ),
        ];

        for (expected, line) in data {
            assert_eq!(expected, parse_map_line(line));
        }
    }

    #[test]
    fn test_parse_memory_range() {
        let (start, size) = parse_memory_range("ffffffffff600000-ffffffffff601000");
        assert_eq!(start, "ffffffffff600000");
        assert_eq!(size, 4);

        let (start, size) = parse_memory_range("7ffc4f0c2000-7ffc4f0e3000");
        assert_eq!(start, "00007ffc4f0c2000");
        assert_eq!(size, 132);
    }

    #[test]
    fn test_parse_perms() {
        assert_eq!("-----", parse_perms("---p"));
        assert_eq!("---s-", parse_perms("---s"));
        assert_eq!("rwx--", parse_perms("rwxp"));
    }

    #[test]
    fn test_parse_filename() {
        assert_eq!("  [ anon ]", parse_filename(""));
        assert_eq!("  [ anon ]", parse_filename("[vvar]"));
        assert_eq!("  [ anon ]", parse_filename("[vdso]"));
        assert_eq!("  [ anon ]", parse_filename("anon_inode:i915.gem"));
        assert_eq!("  [ stack ]", parse_filename("[stack]"));
        assert_eq!(
            "ld-linux-x86-64.so.2",
            parse_filename("/usr/lib/ld-linux-x86-64.so.2")
        );
    }
}
