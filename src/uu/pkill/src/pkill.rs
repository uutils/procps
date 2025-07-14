// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
use clap::{arg, crate_version, Command};
#[cfg(unix)]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
#[cfg(unix)]
use std::io::Error;
#[cfg(unix)]
use uu_pgrep::process::ProcessInformation;
use uu_pgrep::process_matcher;
#[cfg(unix)]
use uucore::{
    error::FromIo,
    show,
    signals::{signal_by_name_or_value, signal_name_by_value},
};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pkill.md");
const USAGE: &str = help_usage!("pkill.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    #[cfg(unix)]
    let mut args = args.collect_ignore();
    #[cfg(target_os = "windows")]
    let args = args.collect_ignore();
    #[cfg(unix)]
    handle_obsolete(&mut args);

    let matches = uu_app().try_get_matches_from(&args)?;
    let settings = process_matcher::get_match_settings(&matches)?;

    #[cfg(unix)]
    let sig_name = signal_name_by_value(settings.signal);
    // Signal does not support converting from EXIT
    // Instead, nix::signal::kill expects Option::None to properly handle EXIT
    #[cfg(unix)]
    let sig: Option<Signal> = if sig_name.is_some_and(|name| name == "EXIT") {
        None
    } else {
        let sig = (settings.signal as i32)
            .try_into()
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
        Some(sig)
    };

    // Collect pids
    let pids = process_matcher::find_matching_pids(&settings)?;

    // Send signal
    // TODO: Implement -q
    #[cfg(unix)]
    let echo = matches.get_flag("echo");
    #[cfg(unix)]
    kill(&pids, sig, echo);

    if matches.get_flag("count") {
        println!("{}", pids.len());
    }

    Ok(())
}

#[cfg(unix)]
fn handle_obsolete(args: &mut [String]) {
    // Sanity check
    if args.len() > 2 {
        // Old signal can only be in the first argument position
        let slice = args[1].as_str();
        if let Some(signal) = slice.strip_prefix('-') {
            // Check if it is a valid signal
            let opt_signal = signal_by_name_or_value(signal);
            if opt_signal.is_some() {
                // Replace with long option that clap can parse
                args[1] = format!("--signal={signal}");
            }
        }
    }
}

#[cfg(unix)]
fn kill(pids: &Vec<ProcessInformation>, sig: Option<Signal>, echo: bool) {
    for pid in pids {
        if let Err(e) = signal::kill(Pid::from_raw(pid.pid as i32), sig) {
            show!(Error::from_raw_os_error(e as i32)
                .map_err_context(|| format!("killing pid {} failed", pid.pid)));
        } else if echo {
            println!(
                "{} killed (pid {})",
                pid.cmdline.split(" ").next().unwrap_or(""),
                pid.pid
            );
        }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .args_override_self(true)
        .args([
            // arg!(-<sig>                    "signal to send (either number or name)"),
            // arg!(-q --queue <value>        "integer value to be sent with the signal"),
            arg!(-e --echo                 "display what is killed"),
        ])
        .args(process_matcher::clap_args(
            "Name of the process to kill",
            false,
        ))
}
