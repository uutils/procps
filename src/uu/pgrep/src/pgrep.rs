// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
pub mod process;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use process::{walk_process, ProcessInformation, Teletype};
use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

static REGEX: OnceLock<Regex> = OnceLock::new();

struct Settings {
    exact: bool,
    full: bool,
    ignore_case: bool,
    inverse: bool,
    newest: bool,
    oldest: bool,
    older: Option<u64>,
    parent: Option<Vec<u64>>,
    runstates: Option<String>,
    terminal: Option<HashSet<Teletype>>,
}

/// # Conceptual model of `pgrep`
///
/// At first, `pgrep` command will check the patterns is legal.
/// In this stage, `pgrep` will construct regex if `--exact` argument was passed.
///
/// Then, `pgrep` will collect all *matched* pids, and filtering them.
/// In this stage `pgrep` command will collect all the pids and its information from __/proc/__
/// file system. At the same time, `pgrep` will construct filters from command
/// line arguments to filter the collected pids. Note that the "-o" and "-n" flag filters works
/// if them enabled and based on general collecting result.
///
/// Last, `pgrep` will construct output format from arguments, and print the processed result.
#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let pattern = try_get_pattern_from(&matches)?;
    REGEX
        .set(Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?)
        .unwrap();

    let settings = Settings {
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
            "no matching criteria specified\nTry `pgrep --help' for more information.",
        ));
    }

    // Collect pids
    let pids = {
        let mut pids = collect_matched_pids(&settings);
        if pids.is_empty() {
            uucore::error::set_exit_code(1);
            pids
        } else {
            process_flag_o_n(&settings, &mut pids)
        }
    };

    // Processing output
    let output = if matches.get_flag("count") {
        format!("{}", pids.len())
    } else {
        let delimiter = matches.get_one::<String>("delimiter").unwrap();

        let formatted: Vec<_> = if matches.get_flag("list-full") {
            pids.into_iter()
                .map(|it| {
                    // pgrep from procps-ng outputs the process name inside square brackets
                    // if /proc/<PID>/cmdline is empty
                    if it.cmdline.is_empty() {
                        format!("{} [{}]", it.pid, it.clone().status().get("Name").unwrap())
                    } else {
                        format!("{} {}", it.pid, it.cmdline)
                    }
                })
                .collect()
        } else if matches.get_flag("list-name") {
            pids.into_iter()
                .map(|it| format!("{} {}", it.pid, it.clone().status().get("Name").unwrap()))
                .collect()
        } else {
            pids.into_iter().map(|it| format!("{}", it.pid)).collect()
        };

        formatted.join(delimiter)
    };

    if !output.is_empty() {
        println!("{}", output);
    };

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

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .args_override_self(true)
        .group(ArgGroup::new("oldest_newest").args(["oldest", "newest", "inverse"]))
        .args([
            arg!(-d     --delimiter <string>    "specify output delimiter")
                .default_value("\n")
                .hide_default_value(true),
            arg!(-l     --"list-name"           "list PID and process name"),
            arg!(-a     --"list-full"           "list PID and full command line"),
            arg!(-v     --inverse               "negates the matching"),
            // arg!(-w     --lightweight           "list all TID"),
            arg!(-c     --count                 "count of matching processes"),
            arg!(-f     --full                  "use full process name to match"),
            // arg!(-g     --pgroup <PGID>     ... "match listed process group IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            // arg!(-G     --group <GID>       ... "match real group IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            arg!(-i     --"ignore-case"         "match case insensitively"),
            arg!(-n     --newest                "select most recently started"),
            arg!(-o     --oldest                "select least recently started"),
            arg!(-O     --older <seconds>       "select where older than seconds")
                .value_parser(clap::value_parser!(u64)),
            arg!(-P     --parent <PPID>         "match only child processes of the given parent")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u64)),
            // arg!(-s     --session <SID>         "match session IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            // arg!(--signal <sig>                 "signal to send (either number or name)"),
            arg!(-t     --terminal <tty>        "match by controlling terminal")
                .value_delimiter(','),
            // arg!(-u     --euid <ID>         ... "match by effective IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            // arg!(-U     --uid <ID>          ... "match by real IDs")
            //     .value_delimiter(',')
            //     .value_parser(clap::value_parser!(u64)),
            arg!(-x     --exact                 "match exactly with the command name"),
            // arg!(-F     --pidfile <file>        "read PIDs from file"),
            // arg!(-L     --logpidfile            "fail if PID file is not locked"),
            arg!(-r     --runstates <state>     "match runstates [D,S,Z,...]"),
            // arg!(-A     --"ignore-ancestors"    "exclude our ancestors from results"),
            // arg!(--cgroup <grp>                 "match by cgroup v2 names")
            //     .value_delimiter(','),
            // arg!(       --ns <PID>              "match the processes that belong to the same namespace as <pid>"),
            // arg!(       --nslist <ns>       ... "list which namespaces will be considered for the --ns option.")
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
