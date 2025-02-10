// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Common process matcher logic shared by pgrep, pkill and pidwait

use std::collections::HashSet;

use clap::{arg, Arg, ArgAction, ArgMatches};
use regex::Regex;
use uucore::error::{UResult, USimpleError};
#[cfg(unix)]
use uucore::{display::Quotable, signals::signal_by_name_or_value};

use crate::process::{walk_process, ProcessInformation, Teletype};

pub struct Settings {
    pub regex: Regex,

    pub exact: bool,
    pub full: bool,
    pub ignore_case: bool,
    pub inverse: bool,
    pub newest: bool,
    pub oldest: bool,
    pub older: Option<u64>,
    pub parent: Option<Vec<u64>>,
    pub runstates: Option<String>,
    pub terminal: Option<HashSet<Teletype>>,
    #[cfg(unix)]
    pub signal: usize,
    pub require_handler: bool,
}

pub fn get_match_settings(matches: &ArgMatches) -> UResult<Settings> {
    let pattern = try_get_pattern_from(matches)?;
    let regex = Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?;

    let settings = Settings {
        regex,
        exact: matches.get_flag("exact"),
        full: matches.get_flag("full"),
        ignore_case: matches.get_flag("ignore-case"),
        inverse: matches.get_flag("inverse"),
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
        #[cfg(unix)]
        signal: parse_signal_value(matches.get_one::<String>("signal").unwrap())?,
        require_handler: matches.get_flag("require-handler"),
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
            format!(
                "no matching criteria specified\n\
                 Try `{} --help' for more information.",
                uucore::util_name()
            ),
        ));
    }

    Ok(settings)
}

pub fn find_matching_pids(settings: &Settings) -> Vec<ProcessInformation> {
    let mut pids = collect_matched_pids(settings);
    #[cfg(unix)]
    if settings.require_handler {
        pids.retain(|pid| {
            let mask =
                u64::from_str_radix(pid.clone().status().get("SigCgt").unwrap(), 16).unwrap();
            mask & (1 << settings.signal) != 0
        });
    }
    if pids.is_empty() {
        uucore::error::set_exit_code(1);
        pids
    } else {
        process_flag_o_n(settings, &mut pids)
    }
}

/// Try to get the pattern from the command line arguments. Returns an empty string if no pattern
/// is specified.
fn try_get_pattern_from(matches: &ArgMatches) -> UResult<String> {
    let pattern = match matches.get_many::<String>("pattern") {
        Some(patterns) if patterns.len() > 1 => {
            return Err(USimpleError::new(
                2,
                format!(
                    "only one pattern can be provided\nTry `{} --help' for more information.",
                    uucore::util_name()
                ),
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

                settings.regex.is_match(want)
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

            if (run_state_matched
                && pattern_matched
                && tty_matched
                && older_matched
                && parent_matched)
                ^ settings.inverse
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
fn parse_signal_value(signal_name: &str) -> UResult<usize> {
    signal_by_name_or_value(signal_name)
        .ok_or_else(|| USimpleError::new(1, format!("Unknown signal {}", signal_name.quote())))
}

#[allow(clippy::cognitive_complexity)]
pub fn clap_args(pattern_help: &'static str, enable_v_flag: bool) -> Vec<Arg> {
    vec![
        if enable_v_flag {
            arg!(-v --inverse          "negates the matching").group("oldest_newest_inverse")
        } else {
            arg!(--inverse             "negates the matching").group("oldest_newest_inverse")
        },
        arg!(-H --"require-handler"    "match only if signal handler is present"),
        arg!(-c --count                "count of matching processes"),
        arg!(-f --full                 "use full process name to match"),
        // arg!(-g --pgroup <PGID>        "match listed process group IDs")
        //     .value_delimiter(',')
        //     .value_parser(clap::value_parser!(u64)),
        // arg!(-G --group <GID>          "match real group IDs")
        //     .value_delimiter(',')
        //     .value_parser(clap::value_parser!(u64)),
        arg!(-i --"ignore-case"        "match case insensitively"),
        arg!(-n --newest               "select most recently started")
            .group("oldest_newest_inverse"),
        arg!(-o --oldest               "select least recently started")
            .group("oldest_newest_inverse"),
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
        Arg::new("pattern")
            .help(pattern_help)
            .action(ArgAction::Append)
            .index(1),
    ]
}
