// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
#[cfg(unix)]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use regex::Regex;
use std::io::Error;
use std::{collections::HashSet, sync::OnceLock};
use uu_pgrep::process::{walk_process, ProcessInformation, Teletype};
#[cfg(unix)]
use uucore::{
    display::Quotable,
    error::FromIo,
    show,
    signals::{signal_by_name_or_value, signal_name_by_value},
};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("pkill.md");
const USAGE: &str = help_usage!("pkill.md");

static REGEX: OnceLock<Regex> = OnceLock::new();

struct Settings {
    exact: bool,
    full: bool,
    ignore_case: bool,
    newest: bool,
    oldest: bool,
    older: Option<u64>,
    parent: Option<Vec<u64>>,
    runstates: Option<String>,
    terminal: Option<HashSet<Teletype>>,
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let mut args = args.collect_ignore();
    #[cfg(unix)]
    let obs_signal = handle_obsolete(&mut args);

    let matches = uu_app().try_get_matches_from(&args)?;

    let pattern = try_get_pattern_from(&matches)?;
    REGEX
        .set(Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?)
        .unwrap();

    let settings = Settings {
        exact: matches.get_flag("exact"),
        full: matches.get_flag("full"),
        ignore_case: matches.get_flag("ignore-case"),
        newest: matches.get_flag("newest"),
        oldest: matches.get_flag("oldest"),
        parent: matches
            .get_many::<u64>("parent")
            .map(|parents| parents.copied().collect()),
        runstates: matches.get_one::<String>("runstates").cloned(),
        older: matches.get_one::<u64>("older").copied(),
        terminal: matches.get_many::<String>("terminal").map(|ttys| {
            ttys.cloned()
                .flat_map(Teletype::try_from)
                .collect::<HashSet<_>>()
        }),
    };

    if (!settings.newest
        && !settings.oldest
        && settings.runstates.is_none()
        && settings.older.is_none()
        && settings.parent.is_none()
        && settings.terminal.is_none())
        && pattern.is_empty()
    {
        return Err(USimpleError::new(
            2,
            "no matching criteria specified\nTry `pkill --help' for more information.",
        ));
    }

    // Parse signal
    #[cfg(unix)]
    let sig_num = if let Some(signal) = obs_signal {
        signal
    } else if let Some(signal) = matches.get_one::<String>("signal") {
        parse_signal_value(signal)?
    } else {
        15_usize //SIGTERM
    };

    #[cfg(unix)]
    let sig_name = signal_name_by_value(sig_num);
    // Signal does not support converting from EXIT
    // Instead, nix::signal::kill expects Option::None to properly handle EXIT
    #[cfg(unix)]
    let sig: Option<Signal> = if sig_name.is_some_and(|name| name == "EXIT") {
        None
    } else {
        let sig = (sig_num as i32)
            .try_into()
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
        Some(sig)
    };

    // Collect pids
    let pids = {
        let mut pids = collect_matched_pids(&settings);
        #[cfg(unix)]
        if matches.get_flag("require-handler") {
            pids.retain(|pid| {
                let mask =
                    u32::from_str_radix(pid.clone().status().get("SigCgt").unwrap(), 16).unwrap();
                mask & (1 << sig_num) != 0
            });
        }
        if pids.is_empty() {
            uucore::error::set_exit_code(1);
            pids
        } else {
            process_flag_o_n(&settings, &mut pids)
        }
    };

    // Send signal
    // TODO: Implement -q
    let echo = matches.get_flag("echo");
    #[cfg(unix)]
    kill(&pids, sig, echo);

    if matches.get_flag("count") {
        println!("{}", pids.len());
    }

    Ok(())
}

/// Try to get the pattern from the command line arguments. Returns an empty string if no pattern
/// is specified.
fn try_get_pattern_from(matches: &ArgMatches) -> UResult<String> {
    let pattern = match matches.get_many::<String>("pattern") {
        Some(patterns) if patterns.len() > 1 => {
            return Err(USimpleError::new(
                2,
                "only one pattern can be provided\nTry `pgrep --help' for more information.",
            ))
        }
        Some(mut patterns) => patterns.next().unwrap(),
        None => return Ok(String::new()),
    };

    let pattern = if matches.get_flag("ignore-case") {
        &pattern.to_lowercase()
    } else {
        pattern
    };

    let pattern = if matches.get_flag("exact") {
        &format!("^{}$", pattern)
    } else {
        pattern
    };

    Ok(pattern.to_string())
}

/// Collect pids with filter construct from command line arguments
fn collect_matched_pids(settings: &Settings) -> Vec<ProcessInformation> {
    // Filtration general parameters
    let filtered: Vec<ProcessInformation> = {
        let mut tmp_vec = Vec::new();

        for mut pid in walk_process().collect::<Vec<_>>() {
            let run_state_matched = match (&settings.runstates, pid.run_state()) {
                (Some(arg_run_states), Ok(pid_state)) => {
                    arg_run_states.contains(&pid_state.to_string())
                }
                (_, Err(_)) => false,
                _ => true,
            };

            let binding = pid.status();
            let name = binding.get("Name").unwrap();
            let name = if settings.ignore_case {
                name.to_lowercase()
            } else {
                name.into()
            };
            let pattern_matched = {
                let want = if settings.exact {
                    // Equals `Name` in /proc/<pid>/status
                    // The `unwrap` operation must succeed
                    // because the REGEX has been verified as correct in `uumain`.
                    &name
                } else if settings.full {
                    // Equals `cmdline` in /proc/<pid>/cmdline
                    &pid.cmdline
                } else {
                    // From manpage:
                    // The process name used for matching is limited to the 15 characters present in the output of /proc/pid/stat.
                    &pid.proc_stat()[..15]
                };

                REGEX.get().unwrap().is_match(want)
            };

            let tty_matched = match &settings.terminal {
                Some(ttys) => ttys.contains(&pid.tty()),
                None => true,
            };

            let arg_older = settings.older.unwrap_or(0);
            let older_matched = pid.start_time().unwrap() >= arg_older;

            // the PPID is the fourth field in /proc/<PID>/stat
            // (https://www.kernel.org/doc/html/latest/filesystems/proc.html#id10)
            let stat = pid.stat();
            let ppid = stat.get(3);
            let parent_matched = match (&settings.parent, ppid) {
                (Some(parents), Some(ppid)) => parents.contains(&ppid.parse::<u64>().unwrap()),
                _ => true,
            };

            if run_state_matched
                && pattern_matched
                && tty_matched
                && older_matched
                && parent_matched
            {
                tmp_vec.push(pid);
            }
        }
        tmp_vec
    };

    filtered
}

/// Sorting pids for flag `-o` and `-n`.
///
/// This function can also be used as a filter to filter out process information.
fn process_flag_o_n(
    settings: &Settings,
    pids: &mut [ProcessInformation],
) -> Vec<ProcessInformation> {
    if settings.oldest || settings.newest {
        pids.sort_by(|a, b| {
            b.clone()
                .start_time()
                .unwrap()
                .cmp(&a.clone().start_time().unwrap())
        });

        let start_time = if settings.newest {
            pids.first().cloned().unwrap().start_time().unwrap()
        } else {
            pids.last().cloned().unwrap().start_time().unwrap()
        };

        // There might be some process start at same time, so need to be filtered.
        let mut filtered = pids
            .iter()
            .filter(|it| (*it).clone().start_time().unwrap() == start_time)
            .collect::<Vec<_>>();

        if settings.newest {
            filtered.sort_by(|a, b| b.pid.cmp(&a.pid));
        } else {
            filtered.sort_by(|a, b| a.pid.cmp(&b.pid));
        }

        vec![filtered.first().cloned().unwrap().clone()]
    } else {
        pids.to_vec()
    }
}

#[cfg(unix)]
fn handle_obsolete(args: &mut Vec<String>) -> Option<usize> {
    // Sanity check
    if args.len() > 2 {
        // Old signal can only be in the first argument position
        let slice = args[1].as_str();
        if let Some(signal) = slice.strip_prefix('-') {
            // Check if it is a valid signal
            let opt_signal = signal_by_name_or_value(signal);
            if opt_signal.is_some() {
                // remove the signal before return
                args.remove(1);
                return opt_signal;
            }
        }
    }
    None
}

#[cfg(unix)]
fn parse_signal_value(signal_name: &str) -> UResult<usize> {
    let optional_signal_value = signal_by_name_or_value(signal_name);
    match optional_signal_value {
        Some(x) => Ok(x),
        None => Err(USimpleError::new(
            1,
            format!("Unknown signal {}", signal_name.quote()),
        )),
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
        .group(ArgGroup::new("oldest_newest").args(["oldest", "newest"]))
        .args([
            // arg!(-<sig>                    "signal to send (either number or name)"),
            arg!(-H --"require-handler"    "match only if signal handler is present"),
            arg!(-q --queue <value>        "integer value to be sent with the signal"),
            arg!(-e --echo                 "display what is killed"),
            arg!(-c --count                "count of matching processes"),
            arg!(-f --full                 "use full process name to match"),
            arg!(-g --pgroup <PGID>        "match listed process group IDs")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(-G --group <GID>          "match real group IDs")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(-i --"ignore-case"        "match case insensitively"),
            arg!(-n --newest               "select most recently started"),
            arg!(-o --oldest               "select least recently started"),
            arg!(-O --older <seconds>      "select where older than seconds")
                .value_parser(clap::value_parser!(u64)),
            arg!(-P --parent <PPID>        "match only child processes of the given parent")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(-s --session <SID>        "match session IDs")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(--signal <sig>            "signal to send (either number or name)"),
            arg!(-t --terminal <tty>       "match by controlling terminal")
                .value_delimiter(','),
            arg!(-u --euid <ID>            "match by effective IDs")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(-U --uid <ID>             "match by real IDs")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            arg!(-x --exact                "match exactly with the command name"),
            arg!(-F --pidfile <file>       "read PIDs from file"),
            arg!(-L --logpidfile           "fail if PID file is not locked"),
            arg!(-r --runstates <state>    "match runstates [D,S,Z,...]"),
            arg!(-A --"ignore-ancestors"   "exclude our ancestors from results"),
            arg!(--cgroup <grp>            "match by cgroup v2 names")
                .value_delimiter(','),
            arg!(--ns <PID>                "match the processes that belong to the same namespace as <pid>"),
            arg!(--nslist <ns>             "list which namespaces will be considered for the --ns option.")
                .value_delimiter(',')
                .value_parser(["ipc", "mnt", "net", "pid", "user", "uts"]),
        ])
        .arg(
            Arg::new("pattern")
                .help("Name of the program to find the PID of")
                .action(ArgAction::Append)
                .index(1),
        )
}
