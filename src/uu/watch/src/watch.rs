// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io::{Error, ErrorKind};
use clap::crate_version;
use clap::{Arg, Command};
use std::process::{Command as SystemCommand, Stdio};
use std::thread::sleep;
use std::time::Duration;
use uucore::{error::UResult, format_usage, help_about, help_usage};
use std::num::{ParseIntError};

const ABOUT: &str = help_about!("watch.md");
const USAGE: &str = help_usage!("watch.md");

fn parse_interval(input: &str) -> Result<Duration, ParseIntError> {
    // Find index where to split string into seconds and nanos
    let index = match input.find(|c: char| c == ',' || c == '.') {
        Some(index) => index,
        None => {
            let seconds: u64 = input.parse()?;
            return Ok(Duration::new(seconds, 0));
        }
    };

    // If the seconds string is empty, set seconds to 0
    let seconds: u64 = if index > 0 { input[..index].parse()? } else { 0 };

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
        }
    };

    loop {
        let output = SystemCommand::new("sh")
            .arg("-c")
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
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
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
