// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::crate_version;
use clap::{Arg, Command};
use std::process::{Command as SystemCommand, Stdio};
use std::thread::sleep;
use std::time::Duration;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("watch.md");
const USAGE: &str = help_usage!("watch.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let command_to_watch = matches
        .get_one::<String>("command")
        .expect("required argument");
    let interval = 2; // TODO matches.get_one::<u64>("interval").map_or(2, |&v| v);

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

        sleep(Duration::from_secs(interval));
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
