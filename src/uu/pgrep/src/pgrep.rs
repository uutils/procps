// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod pid;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use pid::{walk_pid, PidEntry, TerminalType};
use regex::Regex;
use std::{borrow::BorrowMut, cmp::Ordering, collections::HashSet, sync::OnceLock};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

static REGEX: OnceLock<Regex> = OnceLock::new();

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let pattern = collect_arg_patterns(&matches);

    //// Pattern check ////
    let flag_newest = matches.get_flag("newest");
    let flag_oldest = matches.get_flag("oldest");

    if (!flag_newest && !flag_oldest) && pattern.is_empty() {
        return Err(USimpleError::new(
            2,
            "no matching criteria specified\nTry `pgrep --help' for more information.",
        ));
    }

    if pattern.len() > 1 {
        return Err(USimpleError::new(
            2,
            "only one pattern can be provided\nTry `pgrep --help' for more information.",
        ));
    }

    // Verifying regex pattern
    // And put it into static `REGEX`
    if matches.get_flag("full") {
        let regex = Regex::new(pattern.first().unwrap())
            .map_err(|e| USimpleError::new(1, e.to_string()))?;
        REGEX.set(regex).unwrap();
    }

    //// Collect pids ////
    let pids = collect_matched_pids(&matches);

    //// Filtering pids ////
    let pids = if flag_newest || flag_oldest {
        let arg_older = matches.get_one::<u64>("older").unwrap_or(&0);
        filter_oldest_newest(pids, flag_newest, *arg_older)
    } else {
        pids
    };

    if pids.is_empty() {
        uucore::error::set_exit_code(1);
    }

    //// Processing output ////
    let result = || {
        if matches.get_flag("count") {
            return format!("{}", pids.len());
        };

        let delimiter = matches.get_one::<String>("delimiter").unwrap();

        let formatted: Vec<_> = if matches.get_flag("list-full") {
            pids.into_iter()
                .map(|it| format!("{} {}", it.pid, it.cmdline))
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

    let output = result();
    if !output.is_empty() {
        println!("{}", output);
    };

    Ok(())
}

fn collect_arg_patterns(matches: &ArgMatches) -> Vec<String> {
    let should_ignore_case = matches.get_flag("ignore-case");

    let patterns = matches
        .get_many::<String>("pattern")
        .unwrap_or_default()
        .map(|it| it.to_string());

    if should_ignore_case {
        patterns.map(|it| it.to_lowercase()).collect::<Vec<_>>()
    } else {
        patterns.collect()
    }
}

fn collect_matched_pids(matches: &ArgMatches) -> Vec<PidEntry> {
    let binding = String::from("");
    let pattern = matches.get_one::<String>("pattern").unwrap_or(&binding);

    let should_inverse = matches.get_flag("inverse");
    let should_ignore_case = matches.get_flag("ignore-case");

    let flag_full = matches.get_flag("full");
    let flag_exact = matches.get_flag("exact");

    let evaluate = |mut it: PidEntry| {
        let binding = it.status();
        let name = binding.get("Name")?;

        // Processs flag `--ignore-case`
        let name = if should_ignore_case {
            name.to_lowercase()
        } else {
            name.into()
        };

        let name_matched = if flag_full {
            // Equals `Name` in /proc/<pid>/status
            // The `unwrap` operation must succeed
            // because the REGEX has been verified as correct in `uumain`.
            REGEX.get().unwrap().is_match(&name)
        } else if flag_exact {
            // Equals `cmdline` in /proc/<pid>/cmdline
            it.cmdline.eq(pattern)
        } else {
            name.contains(pattern)
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

        if (name_matched && tty_matched) ^ should_inverse {
            Some(it)
        } else {
            None
        }
    };

    let mut result = Vec::new();

    for pid in walk_pid() {
        if let Some(pid) = evaluate(pid) {
            result.push(pid.clone())
        }
    }

    result
}

// Make -o and -n as a group of args
fn filter_oldest_newest(pids: Vec<PidEntry>, flag_newest: bool, arg_older: u64) -> Vec<PidEntry> {
    let mut pids = {
        let mut tmp_vec = Vec::with_capacity(pids.len());
        for mut pid in pids {
            if pid.start_time().unwrap() >= arg_older {
                tmp_vec.push(pid)
            }
        }
        tmp_vec
    };

    pids.sort_by(|a, b| {
        if let (Ok(b), Ok(a)) = (
            b.to_owned().borrow_mut().start_time(),
            a.to_owned().borrow_mut().start_time(),
        ) {
            b.cmp(&a)
        } else {
            Ordering::Equal
        }
    });

    let mut entry = if flag_newest {
        pids.first()
    } else {
        pids.last()
    }
    .cloned()
    .unwrap();

    let sort = |start_time: u64| {
        let mut result = pids
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

    vec![sort(entry.start_time().unwrap())
        .first()
        .unwrap()
        .clone()
        .clone()]
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
