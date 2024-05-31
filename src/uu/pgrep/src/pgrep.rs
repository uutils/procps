// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod pid;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use pid::{walk_pid, PidEntry};
use std::{borrow::BorrowMut, cmp::Ordering};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let patterns = collect_arg_patterns(&matches);

    // Some(()) => Detected
    // None => Go on
    if handle_oldest_newest(&matches, &patterns).is_some() {
        return Ok(());
    }

    handle_normal_pid_collect(&matches, &patterns);

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

    walk_pid()
        .filter(move |it| {
            let binding = it.to_owned().borrow_mut().status();
            let Some(name) = binding.get("Name") else {
                return false;
            };

            // Processs flag `--ignore-case`
            let name = if should_ignore_case {
                name.to_lowercase()
            } else {
                name.into()
            };

            patterns.iter().any(|it| name.contains(it)) ^ should_inverse
        })
        .collect::<Vec<_>>()
}

// Make -o and -n as a group of args
fn handle_oldest_newest(matches: &ArgMatches, patterns: &[String]) -> Option<()> {
    let flag_newest = matches.get_flag("newest");
    let flag_oldest = matches.get_flag("oldest");

    if flag_newest != flag_oldest {
        // Only accept one pattern.
        if !patterns.is_empty() && patterns.len() != 1 {
            println!("{:?}", patterns);
            println!("pgrep: only one pattern can be provided");
            return Some(());
        }

        // Processing pattern
        let mut result = if patterns.len() == 1 {
            collect_pid(matches, patterns)
        } else {
            walk_pid().collect()
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

        let sort = |start_time: u64| {
            let mut result = result
                .iter()
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

        let mut entry = if flag_newest {
            result.first()
        } else {
            result.last()
        }
        .expect("empty pid list")
        .to_owned();

        println!("{}", sort(entry.start_time().unwrap()).first().unwrap().pid);
        return Some(());
    }

    None
}

fn handle_normal_pid_collect(matches: &ArgMatches, patterns: &[String]) {
    let delimiter = matches.get_one::<String>("delimiter").unwrap();

    let result = collect_pid(matches, patterns);

    let flag_list_name = matches.get_flag("list-name");
    let flag_list_full = matches.get_flag("list-full");

    let flag_count = matches.get_flag("count");
    if flag_count {
        println!("{}", result.len());
    } else {
        // Normal output
        let result = result
            .iter()
            .map(|it| {
                if flag_list_full {
                    format!("{} {}", it.pid, it.cmdline)
                } else if flag_list_name {
                    let name = it
                        .to_owned()
                        .borrow_mut()
                        .status()
                        .get("Name")
                        .cloned()
                        .unwrap_or_default();
                    format!("{} {}", it.pid, name)
                } else {
                    format!("{}", it.pid)
                }
            })
            .collect::<Vec<_>>();
        println!("{}", result.join(delimiter));
    }
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .arg_required_else_help(true)
        .override_usage(format_usage(USAGE))
        .group(ArgGroup::new("oldest_newest").args(["oldest", "newest"]))
        .args([
            arg!(-d     --delimiter <string>    "specify output delimiter")
                .default_value("\n")
                .hide_default_value(true),
            arg!(-l     --"list-name"           "list PID and process name")
                .action(ArgAction::SetTrue),
            arg!(-a     --"list-full"           "list PID and full command line")
                .action(ArgAction::SetTrue),
            arg!(-v     --inverse               "negates the matching").action(ArgAction::SetTrue),
            // arg!(-w     --lightweight           "list all TID"),
            arg!(-c     --count                 "count of matching processes"),
            // arg!(-f     --full                  "use full process name to match"),
            // arg!(-g     --pgroup <PGID>     ... "match listed process group IDs"),
            // arg!(-G     --group <GID>       ... "match real group IDs"),
            arg!(-i     --"ignore-case"         "match case insensitively")
                .action(ArgAction::SetTrue),
            arg!(-n     --newest                "select most recently started")
                .action(ArgAction::SetTrue),
            arg!(-o     --oldest                "select least recently started")
                .action(ArgAction::SetTrue),
            // arg!(-O     --older <seconds>       "select where older than seconds"),
            // arg!(-P     --parent <PPID>         "match only child processes of the given parent"),
            // arg!(-s     --session <SID>         "match session IDs"),
            // arg!(-t     --terminal <tty>        "match by controlling terminal"),
            // arg!(-u     --euid <ID>         ... "match by effective IDs"),
            // arg!(-U     --uid <ID>          ... "match by real IDs"),
            // arg!(-x     --exact                 "match exactly with the command name"),
            // arg!(-F     --pidfile <file>        "read PIDs from file"),
            // arg!(-L     --logpidfile            "fail if PID file is not locked"),
            // arg!(-r     --runstates <state>     "match runstates [D,S,Z,...]"),
            // arg!(       --ns <PID>              "match the processes that belong to the same namespace as <pid>"),
            // arg!(       --nslist <ns>       ... "list which namespaces will be considered for the --ns option."),
        ])
        .arg(
            Arg::new("pattern")
                .help("Name of the program to find the PID of")
                .index(1),
        )
}
