// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::crate_version;
use clap::{Arg, Command};
use std::io::{Error, ErrorKind};
use std::num::ParseIntError;
use std::process::{Command as SystemCommand, Stdio};
use std::thread::sleep;
use std::time::Duration;
use uucore::error::UResult;

fn parse_interval(input: &str) -> Result<Duration, ParseIntError> {
    // Find index where to split string into seconds and nanos
    let Some(index) = input.find([',', '.']) else {
        let seconds: u64 = input.parse()?;

        return if seconds == 0 {
            Ok(Duration::from_millis(100))
        } else {
            Ok(Duration::new(seconds, 0))
        };
    };

    // If the seconds string is empty, set seconds to 0
    let seconds: u64 = if index > 0 {
        input[..index].parse()?
    } else {
        0
    };

    let nanos_string = &input[index + 1..];
    let nanos: u32 = match nanos_string.len() {
        // If nanos string is empty, set nanos to 0
        0 => 0,
        1..=9 => {
            let nanos: u32 = nanos_string.parse()?;
            nanos * 10u32.pow((9 - nanos_string.len()) as u32)
        }
        _ => {
            // This parse is used to validate if the rest of the string is indeed numeric
            if nanos_string.find(|c: char| !c.is_numeric()).is_some() {
                "a".parse::<u8>()?;
            }
            // We can have only 9 digits of accuracy, trim the rest
            nanos_string[..9].parse()?
        }
    };

    let duration = Duration::new(seconds, nanos);
    // Minimum duration of sleep to 0.1 s
    Ok(std::cmp::max(duration, Duration::from_millis(100)))
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let command_to_watch = matches
        .get_one::<String>("command")
        .expect("required argument");
    let interval = match matches.get_one::<String>("interval") {
        None => Duration::from_secs(2),
        Some(input) => match parse_interval(input) {
            Ok(interval) => interval,
            Err(_) => {
                return Err(Box::from(Error::new(
                    ErrorKind::InvalidInput,
                    format!("watch: failed to parse argument: '{input}': Invalid argument"),
                )));
            }
        },
    };

    loop {
        #[cfg(windows)]
        let mut command =
            SystemCommand::new(std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()));
        #[cfg(not(windows))]
        let mut command = SystemCommand::new("sh");

        #[cfg(windows)]
        command.arg("/c");
        #[cfg(not(windows))]
        command.arg("-c");

        let output = command
            .arg(command_to_watch)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;

        if !output.status.success() {
            eprintln!("watch: command failed: {:?}", output.status);
            break;
        }

        sleep(interval);
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Execute a program periodically, showing output fullscreen")
        .override_usage("watch [options] command")
        .infer_long_args(true)
        .arg(
            Arg::new("command")
                .required(true)
                .help("Command to be executed"),
        )
        .arg(
            Arg::new("interval")
                .short('n')
                .long("interval")
                .help("Seconds to wait between updates")
                .default_value("2")
                .env("WATCH_INTERVAL")
                .value_name("SECONDS"),
        )
        .arg(
            Arg::new("beep")
                .short('b')
                .long("beep")
                .help("Beep if command has a non-zero exit"),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .help("Interpret ANSI color and style sequences"),
        )
        .arg(
            Arg::new("no-color")
                .short('C')
                .long("no-color")
                .help("Do not interpret ANSI color and style sequences"),
        )
        .arg(
            Arg::new("differences")
                .short('d')
                .long("differences")
                .value_name("permanent")
                .help("Highlight changes between updates"),
        )
        .arg(
            Arg::new("errexit")
                .short('e')
                .long("errexit")
                .help("Exit if command has a non-zero exit"),
        )
        .arg(
            Arg::new("chgexit")
                .short('g')
                .long("chgexit")
                .help("Exit when output from command changes"),
        )
        .arg(
            Arg::new("equexit")
                .short('q')
                .long("equexit")
                .value_name("CYCLES")
                .help("Exit when output from command does not change"),
        )
        .arg(
            Arg::new("precise")
                .short('p')
                .long("precise")
                .help("Attempt to run command in precise intervals"),
        )
        .arg(
            Arg::new("no-rerun")
                .short('r')
                .long("no-rerun")
                .help("Do not rerun program on window resize"),
        )
        .arg(
            Arg::new("no-title")
                .short('t')
                .long("no-title")
                .help("Turn off header"),
        )
        .arg(
            Arg::new("no-wrap")
                .short('w')
                .long("no-wrap")
                .help("Turn off line wrapping"),
        )
        .arg(
            Arg::new("exec")
                .short('x')
                .long("exec")
                .help("Pass command to exec instead of 'sh -c'"),
        )
}

#[cfg(test)]
mod parse_interval_tests {
    use super::*;

    #[test]
    fn test_comma_parse() {
        let interval = parse_interval("1,5");
        assert_eq!(Ok(Duration::from_millis(1500)), interval);
    }

    #[test]
    fn test_different_nanos_length() {
        let interval = parse_interval("1.12345");
        assert_eq!(Ok(Duration::new(1, 123450000)), interval);
        let interval = parse_interval("1.1234");
        assert_eq!(Ok(Duration::new(1, 123400000)), interval);
    }

    #[test]
    fn test_period_parse() {
        let interval = parse_interval("1.5");
        assert_eq!(Ok(Duration::from_millis(1500)), interval);
    }

    #[test]
    fn test_empty_seconds_interval() {
        let interval = parse_interval(".5");
        assert_eq!(Ok(Duration::from_millis(500)), interval);
    }

    #[test]
    fn test_seconds_only() {
        let interval = parse_interval("7");
        assert_eq!(Ok(Duration::from_secs(7)), interval);
    }

    #[test]
    fn test_empty_nanoseconds_interval() {
        let interval = parse_interval("1.");
        assert_eq!(Ok(Duration::from_millis(1000)), interval);
    }

    #[test]
    fn test_too_many_nanos() {
        let interval = parse_interval("1.00000000009");
        assert_eq!(Ok(Duration::from_secs(1)), interval);
    }

    #[test]
    fn test_invalid_nano() {
        let interval = parse_interval("1.00000000000a");
        assert!(interval.is_err())
    }

    #[test]
    fn test_minimum_seconds() {
        let interval = parse_interval("0");
        assert_eq!(Ok(Duration::from_millis(100)), interval);
    }

    #[test]
    fn test_minimum_nanos() {
        let interval = parse_interval("0.0");
        assert_eq!(Ok(Duration::from_millis(100)), interval);
    }
}
