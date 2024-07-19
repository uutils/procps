// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
pub mod process;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use process::{walk_process, ProcessInformation, TerminalType};
use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

static REGEX: OnceLock<Regex> = OnceLock::new();

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

    // Pattern check
    let flag_newest = matches.get_flag("newest");
    let flag_oldest = matches.get_flag("oldest");

    if (!flag_newest && !flag_oldest) && pattern.is_empty() {
        return Err(USimpleError::new(
            2,
            "no matching criteria specified\nTry `pgrep --help' for more information.",
        ));
    }

    // Collect pids
    let pids = {
        let mut pids = collect_matched_pids(&matches);
        if pids.is_empty() {
            uucore::error::set_exit_code(1);
            pids
        } else {
            process_flag_o_n(&matches, &mut pids)
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
fn collect_matched_pids(matches: &ArgMatches) -> Vec<ProcessInformation> {
    let should_inverse = matches.get_flag("inverse");
    let should_ignore_case = matches.get_flag("ignore-case");

    let flag_full = matches.get_flag("full");
    let flag_exact = matches.get_flag("exact");

    // Filtration general parameters
    let filtered: Vec<ProcessInformation> = {
        let mut tmp_vec = Vec::new();

        for mut pid in walk_process().collect::<Vec<_>>() {
            let run_state_matched =
                match (matches.get_one::<String>("runstates"), (pid).run_state()) {
                    (Some(arg_run_states), Ok(pid_state)) => {
                        arg_run_states.contains(&pid_state.to_string())
                    }
                    _ => true,
                };

            let binding = pid.status();
            let name = binding.get("Name").unwrap();
            let name = if should_ignore_case {
                name.to_lowercase()
            } else {
                name.into()
            };
            let pattern_matched = {
                let want = if flag_exact {
                    // Equals `Name` in /proc/<pid>/status
                    // The `unwrap` operation must succeed
                    // because the REGEX has been verified as correct in `uumain`.
                    &name
                } else if flag_full {
                    // Equals `cmdline` in /proc/<pid>/cmdline
                    &pid.cmdline
                } else {
                    // From manpage:
                    // The process name used for matching is limited to the 15 characters present in the output of /proc/pid/stat.
                    &pid.proc_stat()[..15]
                };

                REGEX.get().unwrap().is_match(want)
            };

            let tty_matched = match matches.get_many::<String>("terminal") {
                Some(ttys) => {
                    // convert from input like `pts/0`
                    let ttys = ttys
                        .cloned()
                        .flat_map(TerminalType::try_from)
                        .collect::<HashSet<_>>();
                    match pid.ttys() {
                        Ok(value) => value.iter().any(|it| ttys.contains(it)),
                        Err(_) => false,
                    }
                }
                None => true,
            };

            let arg_older = matches.get_one::<u64>("older").unwrap_or(&0);
            let older_matched = pid.start_time().unwrap() >= *arg_older;

            if (run_state_matched && pattern_matched && tty_matched && older_matched)
                ^ should_inverse
            {
                tmp_vec.push(pid)
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
    matches: &ArgMatches,
    pids: &mut [ProcessInformation],
) -> Vec<ProcessInformation> {
    let flag_oldest = matches.get_flag("oldest");
    let flag_newest = matches.get_flag("newest");

    if flag_oldest || flag_newest {
        pids.sort_by(|a, b| {
            b.clone()
                .start_time()
                .unwrap()
                .cmp(&a.clone().start_time().unwrap())
        });

        let start_time = if flag_newest {
            pids.first().cloned().unwrap().start_time().unwrap()
        } else {
            pids.last().cloned().unwrap().start_time().unwrap()
        };

        // There might be some process start at same time, so need to be filtered.
        let mut filtered = pids
            .iter()
            .filter(|it| (*it).clone().start_time().unwrap() == start_time)
            .collect::<Vec<_>>();

        if flag_newest {
            filtered.sort_by(|a, b| b.pid.cmp(&a.pid))
        } else {
            filtered.sort_by(|a, b| a.pid.cmp(&b.pid))
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
            // arg!(-g     --pgroup <PGID>     ... "match listed process group IDs"),
            // arg!(-G     --group <GID>       ... "match real group IDs"),
            arg!(-i     --"ignore-case"         "match case insensitively"),
            arg!(-n     --newest                "select most recently started"),
            arg!(-o     --oldest                "select least recently started"),
            arg!(-O     --older <seconds>       "select where older than seconds")
                .num_args(0..)
                .default_value("0")
                .value_parser(clap::value_parser!(u64)),
            // arg!(-P     --parent <PPID>         "match only child processes of the given parent"),
            // arg!(-s     --session <SID>         "match session IDs"),
            arg!(-t     --terminal <tty>        "match by controlling terminal")
                .action(ArgAction::Append),
            // arg!(-u     --euid <ID>         ... "match by effective IDs"),
            // arg!(-U     --uid <ID>          ... "match by real IDs"),
            arg!(-x     --exact                 "match exactly with the command name"),
            // arg!(-F     --pidfile <file>        "read PIDs from file"),
            // arg!(-L     --logpidfile            "fail if PID file is not locked"),
            arg!(-r     --runstates <state>     "match runstates [D,S,Z,...]"),
            // arg!(       --ns <PID>              "match the processes that belong to the same namespace as <pid>"),
            // arg!(       --nslist <ns>       ... "list which namespaces will be considered for the --ns option."),
        ])
        .arg(
            Arg::new("pattern")
                .help("Name of the program to find the PID of")
                .action(ArgAction::Append)
                .index(1),
        )
}
