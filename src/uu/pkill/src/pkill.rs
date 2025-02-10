// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, Command};
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
    let pids = process_matcher::find_matching_pids(&settings);

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
                args[1] = format!("--signal={}", signal);
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
        .group(ArgGroup::new("oldest_newest").args(["oldest", "newest", "inverse"]))
        .args([
            // arg!(-<sig>                    "signal to send (either number or name)"),
            arg!(-H --"require-handler"    "match only if signal handler is present"),
            // arg!(-q --queue <value>        "integer value to be sent with the signal"),
            arg!(-e --echo                 "display what is killed"),
            arg!(--inverse                 "negates the matching"),
            arg!(-c --count                "count of matching processes"),
            arg!(-f --full                 "use full process name to match"),
            // arg!(-g --pgroup <PGID>        "match listed process group IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            // arg!(-G --group <GID>          "match real group IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            arg!(-i --"ignore-case"        "match case insensitively"),
            arg!(-n --newest               "select most recently started"),
            arg!(-o --oldest               "select least recently started"),
            arg!(-O --older <seconds>      "select where older than seconds")
                .value_parser(clap::value_parser!(u64)),
            arg!(-P --parent <PPID>        "match only child processes of the given parent")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            // arg!(-s --session <SID>        "match session IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            arg!(--signal <sig>            "signal to send (either number or name)")
                .default_value("SIGTERM"),
            arg!(-t --terminal <tty>       "match by controlling terminal").value_delimiter(','),
            // arg!(-u --euid <ID>            "match by effective IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            // arg!(-U --uid <ID>             "match by real IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            arg!(-x --exact                "match exactly with the command name"),
            // arg!(-F --pidfile <file>       "read PIDs from file"),
            // arg!(-L --logpidfile           "fail if PID file is not locked"),
            arg!(-r --runstates <state>    "match runstates [D,S,Z,...]"),
            // arg!(-A --"ignore-ancestors"   "exclude our ancestors from results"),
            // arg!(--cgroup <grp>            "match by cgroup v2 names")
            //     .value_delimiter(','),
            // arg!(--ns <PID>                "match the processes that belong to the same namespace as <pid>"),
            // arg!(--nslist <ns>             "list which namespaces will be considered for the --ns option.")
            //     .value_delimiter(',')
            //     .value_parser(["ipc", "mnt", "net", "pid", "user", "uts"]),
        ])
        .arg(
            Arg::new("pattern")
                .help("Name of the program to find the PID of")
                .action(ArgAction::Append)
                .index(1),
        )
}
