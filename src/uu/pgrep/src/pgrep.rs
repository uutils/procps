// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod pid;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use pid::{walk_pid, PidEntry, TerminalType};
use std::{borrow::BorrowMut, cmp::Ordering, collections::HashSet};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let patterns = collect_arg_patterns(&matches);

    let Some(result) =
        collect_oldest_newest(&matches, &patterns).or(collect_normal_pid(&matches, &patterns))
    else {
        println!("pgrep: no matching criteria specified");
        println!("Try `pgrep --help' for more information");
        return Ok(());
    };

    // Just processsing output format here
    let result = || {
        if matches.get_flag("count") {
            return format!("{}", result.len());
        };

        let delimiter = matches.get_one::<String>("delimiter").unwrap();

        let formatted: Vec<_> = if matches.get_flag("list-full") {
            result
                .into_iter()
                .map(|it| format!("{} {}", it.pid, it.cmdline))
                .collect()
        } else if matches.get_flag("list-name") {
            result
                .into_iter()
                .map(|it| format!("{} {}", it.pid, it.clone().status().get("Name").unwrap()))
                .collect()
        } else {
            result.into_iter().map(|it| format!("{}", it.pid)).collect()
        };

        formatted.join(delimiter)
    };

    let result = result();
    if !result.is_empty() {
        println!("{}", result);
    };

    Ok(())
}

fn collect_arg_patterns(matches: &ArgMatches) -> Vec<String> {
    let should_ignore_case = matches.get_flag("ignore-case");

    let programs = matches
        .get_many::<String>("pattern")
        .unwrap_or_default()
        .map(|it| it.to_string());

    if should_ignore_case {
        programs.map(|it| it.to_lowercase()).collect::<Vec<_>>()
    } else {
        programs.collect()
    }
}

fn collect_pid(matches: &ArgMatches, patterns: &[String]) -> Vec<PidEntry> {
    let should_inverse = matches.get_flag("inverse");
    let should_ignore_case = matches.get_flag("ignore-case");

    let flag_full = matches.get_flag("full");
    let flag_exact = matches.get_flag("exact");

    let predicate_closure = |mut it: PidEntry| {
        let binding = it.status();
        let name = binding.get("Name")?;

        // Processs flag `--ignore-case`
        let name = if should_ignore_case {
            name.to_lowercase()
        } else {
            name.into()
        };

        let mut iter = patterns.iter();

        let name_matched = if flag_full {
            // Equals `Name` in /proc/<pid>/status
            iter.any(|it| name.eq(it))
        } else if flag_exact {
            // Equals `cmdline` in /proc/<pid>/cmdline
            iter.any(|s| it.cmdline.eq(s))
        } else {
            // Contains `cmdline` in /proc/<pid>/cmdline
            iter.any(|it| name.contains(it))
        };

        let tty_matched = if let Some(ttys) = matches.get_many::<String>("terminal") {
            // convert from input like `pts/0`
            let ttys = ttys
                .cloned()
                .flat_map(TerminalType::try_from)
                .collect::<HashSet<_>>();

            if let Ok(value) = it.ttys() {
                value.iter().any(|it| ttys.contains(it))
            } else {
                false
            }
        } else {
            true
        };

        if (name_matched || tty_matched) ^ should_inverse {
            Some(it)
        } else {
            None
        }
    };

    let mut result = Vec::new();

    for ele in walk_pid() {
        if let Some(ele) = predicate_closure(ele) {
            result.push(ele)
        }
    }

    result
}

// Make -o and -n as a group of args
fn collect_oldest_newest(matches: &ArgMatches, patterns: &[String]) -> Option<Vec<PidEntry>> {
    let flag_newest = matches.get_flag("newest");
    let flag_oldest = matches.get_flag("oldest");
    let arg_older = matches.get_one::<u64>("older").unwrap();

    if flag_newest != flag_oldest {
        // Only accept one pattern.
        if !patterns.is_empty() && patterns.len() != 1 {
            println!("pgrep: only one pattern can be provided");
            return None;
        }

        // Processing pattern
        let result = collect_pid(matches, patterns);

        let mut result = {
            let mut vec = Vec::with_capacity(result.len());
            for mut ele in result {
                if ele.start_time().unwrap() >= *arg_older {
                    vec.push(ele)
                }
            }
            vec
        };

        result.sort_by(|a, b| {
            if let (Ok(b), Ok(a)) = (
                b.to_owned().borrow_mut().start_time(),
                a.to_owned().borrow_mut().start_time(),
            ) {
                b.cmp(&a)
            } else {
                Ordering::Equal
            }
        });

        // Check is empty
        if result.is_empty() {
            return Some(Vec::new());
        }

        let mut entry = if flag_newest {
            result.first()
        } else {
            result.last()
        }
        .cloned()
        .unwrap();

        let sort = |start_time: u64| {
            let mut result = result
                .into_iter()
                .filter(|it| (*it).to_owned().borrow_mut().start_time().is_ok())
                .filter(move |it| (*it).to_owned().borrow_mut().start_time().unwrap() == start_time)
                .collect::<Vec<_>>();

            result.sort_by(|a, b| {
                if let (Ok(b), Ok(a)) = (
                    (*b).to_owned().borrow_mut().start_time(),
                    (*a).to_owned().borrow_mut().start_time(),
                ) {
                    b.cmp(&a)
                } else {
                    Ordering::Equal
                }
            });

            result
        };

        // TODO: make sure it can be multiple result
        return Some(vec![sort(entry.start_time().unwrap())
            .first()
            .unwrap()
            .clone()
            .clone()]);
    }

    None
}

fn collect_normal_pid(matches: &ArgMatches, patterns: &[String]) -> Option<Vec<PidEntry>> {
    let result = collect_pid(matches, patterns);

    // Normal output
    let result = result.into_iter().collect::<Vec<_>>();

    Some(result)
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .arg_required_else_help(true)
        .override_usage(format_usage(USAGE))
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
            // arg!(-r     --runstates <state>     "match runstates [D,S,Z,...]"),
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
