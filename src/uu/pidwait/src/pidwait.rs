// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, Command};
use std::time::Duration;
use uu_pgrep::process_matcher;
use uucore::{error::UResult, format_usage, help_about, help_usage};
use uu_pgrep::process::ProcessInformation;

const ABOUT: &str = help_about!("pidwait.md");
const USAGE: &str = help_usage!("pidwait.md");

mod platform;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = process_matcher::get_match_settings(&matches)?;
    let mut proc_infos = process_matcher::find_matching_pids(&settings)?;

    // For empty result
    if proc_infos.is_empty() {
        uucore::error::set_exit_code(1);
    }

    // Process outputs
    if matches.get_flag("count") {
        println!("{}", proc_infos.len());
    }

    if matches.get_flag("echo") {
        if settings.newest || settings.oldest {
            for ele in &proc_infos {
                println!("waiting for  (pid {})", ele.pid);
            }
        } else {
            for ele in proc_infos.iter_mut() {
                println!("waiting for {} (pid {})", ele.status()["Name"], ele.pid);
            }
        }
    }

    // It should be fine to reserve a `timeout` parameter for future use.
    wait(&proc_infos, None)?;

    Ok(())
}

pub(crate) fn wait(procs: &[ProcessInformation], timeout: Option<Duration>) -> Result<Option<()>, std::io::Error> {
    if !procs.is_empty() {
        platform::wait(procs, timeout)
    } else {
        Ok(None)
    }
}

pub fn uu_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([arg!(-e --echo                      "display PIDs before waiting")])
        .args(process_matcher::clap_args(
            "Name of the program to wait for",
            true,
        ))
}
