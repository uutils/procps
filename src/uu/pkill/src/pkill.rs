// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
use clap::{arg, crate_version, value_parser, Command};
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
use uucore::error::UResult;
#[cfg(unix)]
use uucore::{
    error::FromIo,
    show,
    signals::{signal_by_name_or_value, signal_name_by_value},
};

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
            .map_err(|e| Error::from_raw_os_error(e as i32))?;
        Some(sig)
    };

    // Collect pids
    let pids = process_matcher::find_matching_pids(&settings)?;

    // Send signal
    #[cfg(unix)]
    {
        let echo = matches.get_flag("echo");
        let queue = matches.get_one::<u32>("queue").cloned();

        kill(&pids, sig, queue, echo);
    }

    if matches.get_flag("count") {
        println!("{}", pids.len());
    }

    Ok(())
}

#[cfg(unix)]
fn handle_obsolete(args: &mut [String]) {
    for arg in &mut args[1..] {
        if let Some(signal) = arg.strip_prefix('-') {
            // Check if it is a valid signal
            let opt_signal = signal_by_name_or_value(signal);
            if opt_signal.is_some() {
                // Replace with long option that clap can parse
                *arg = format!("--signal={signal}");
            }
        }
    }
}

// Not contains in libc
#[cfg(target_os = "linux")]
extern "C" {
    fn sigqueue(
        pid: uucore::libc::pid_t,
        sig: uucore::libc::c_int,
        val: uucore::libc::sigval,
    ) -> uucore::libc::c_int;
}

#[cfg(unix)]
#[allow(unused_variables)]
fn kill(pids: &Vec<ProcessInformation>, sig: Option<Signal>, queue: Option<u32>, echo: bool) {
    for pid in pids {
        #[cfg(target_os = "linux")]
        let result = if let Some(queue) = queue {
            let v = unsafe {
                sigqueue(
                    pid.pid as i32,
                    sig.map_or(0, |s| s as uucore::libc::c_int),
                    uucore::libc::sigval {
                        sival_ptr: queue as usize as *mut uucore::libc::c_void,
                    },
                )
            };
            nix::errno::Errno::result(v).map(drop)
        } else {
            signal::kill(Pid::from_raw(pid.pid as i32), sig)
        };
        #[cfg(not(target_os = "linux"))]
        let result = signal::kill(Pid::from_raw(pid.pid as i32), sig);
        if let Err(e) = result {
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
        .about("Kills processes based on name and other attributes")
        .override_usage("pkill [options] <pattern>")
        .args_override_self(true)
        .args([
            // arg!(-<sig>                    "signal to send (either number or name)"),
            arg!(-q --queue <value>        "integer value to be sent with the signal")
                .value_parser(value_parser!(u32)),
            arg!(-e --echo                 "display what is killed"),
        ])
        .args(process_matcher::clap_args(
            "Name of the process to kill",
            false,
        ))
}
