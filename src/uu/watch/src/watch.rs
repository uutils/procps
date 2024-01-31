// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{Arg, ArgAction, Command};
use clap::{crate_version};
use std::process::{Command as SystemCommand, Stdio};
use std::thread::sleep;
use std::time::Duration;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("watch.md");
const USAGE: &str = help_usage!("watch.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let command_to_watch = matches.get_one::<String>("command").expect("required argument");
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
                .default_value("2")
                .help("Seconds to wait between updates")
                .action(ArgAction::Set),
        )
}
