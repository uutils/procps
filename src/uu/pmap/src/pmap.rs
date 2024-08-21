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
        let (memory_range, rest) = line.split_once(' ').expect("line should contain ' '");
        let (start_address, size_in_kb) = parse_memory_range(memory_range);

        let (perms, rest) = rest.split_once(' ').expect("line should contain 2nd ' '");
        let perms = parse_perms(perms);

        let cmd: String = rest.split_whitespace().skip(3).collect();

        println!("{start_address} {size_in_kb:>6}K {perms} {cmd}");
    }

    Ok(())
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
}
