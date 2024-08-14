// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, Arg, ArgAction, ArgMatches, Command};
use regex::Regex;
use std::{collections::HashSet, env, sync::OnceLock};
use uu_pgrep::process::{walk_process, ProcessInformation, RunState, Teletype};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};
use wait::wait;

mod wait;

const ABOUT: &str = help_about!("pidwait.md");
const USAGE: &str = help_usage!("pidwait.md");

static REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Debug)]
struct Settings {
    echo: bool,
    count: bool,
    full: bool,
    ignore_case: bool,
    newest: bool,
    oldest: bool,
    older: Option<u64>,
    terminal: Option<HashSet<Teletype>>,
    exact: bool,
    runstates: Option<RunState>,
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = Settings {
        echo: matches.get_flag("echo"),
        count: matches.get_flag("count"),
        full: matches.get_flag("full"),
        ignore_case: matches.get_flag("ignore-case"),
        newest: matches.get_flag("newest"),
        oldest: matches.get_flag("oldest"),
        older: matches.get_one::<u64>("older").copied(),
        terminal: matches
            .get_many::<Teletype>("terminal")
            .map(|it| it.cloned().collect()),
        exact: matches.get_flag("exact"),
        runstates: matches.get_one::<RunState>("runstates").cloned(),
    };

    let pattern = initialize_pattern(&matches, &settings)?;
    REGEX
        .set(Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?)
        .unwrap();

    if (!settings.newest
        && !settings.oldest
        && settings.runstates.is_none()
        && settings.older.is_none()
        && settings.terminal.is_none())
        && pattern.is_empty()
    {
        return Err(USimpleError::new(
            2,
            "no matching criteria specified\nTry `pidwait --help' for more information.",
        ));
    }

    let mut proc_infos = collect_proc_infos(&settings);

    // For empty result
    if proc_infos.is_empty() {
        uucore::error::set_exit_code(1);
    }

    // Process outputs
    if settings.count {
        println!("{}", proc_infos.len())
    }

    if settings.echo {
        for ele in proc_infos.iter_mut() {
            println!("waiting for {} (pid {})", ele.status()["Name"], ele.pid)
        }
    }

    wait(&proc_infos);

    Ok(())
}

fn initialize_pattern(matches: &ArgMatches, settings: &Settings) -> UResult<String> {
    let pattern = match matches.get_many::<String>("pattern") {
        Some(patterns) if patterns.len() > 1 => {
            return Err(USimpleError::new(
                2,
                "only one pattern can be provided\nTry `pidwait --help' for more information.",
            ))
        }
        Some(mut patterns) => patterns.next().unwrap(),
        None => return Ok(String::new()),
    };

    let pattern = if settings.ignore_case {
        &pattern.to_lowercase()
    } else {
        pattern
    };

    let pattern = if settings.exact {
        &format!("^{}$", pattern)
    } else {
        pattern
    };

    if !settings.full && pattern.len() >= 15 {
        const MSG_0: &str= "pidwait: pattern that searches for process name longer than 15 characters will result in zero matches";
        const MSG_1: &str = "Try `pidwait -f' option to match against the complete command line.";
        return Err(USimpleError::new(1, format!("{MSG_0}\n{MSG_1}")));
    }

    Ok(pattern.to_string())
}

fn collect_proc_infos(settings: &Settings) -> Vec<ProcessInformation> {
    // Process pattern
    let proc_infos = {
        let mut temp = Vec::new();
        for mut it in walk_process() {
            let matched = {
                let binding = it.status();
                let name = binding.get("Name").unwrap();
                let name = if settings.ignore_case {
                    name.to_lowercase()
                } else {
                    name.into()
                };

                let want = if settings.exact {
                    &name
                } else if settings.full {
                    &it.cmdline
                } else {
                    &it.proc_stat()[..15]
                };

                REGEX.get().unwrap().is_match(want)
            };
            if matched {
                temp.push(it)
            }
        }
        temp
    };

    // Process `-O`
    let mut proc_infos = {
        let mut temp: Vec<ProcessInformation> = Vec::new();
        let older = settings.older.unwrap_or_default();
        for mut proc_info in proc_infos {
            if proc_info.start_time().unwrap() >= older {
                temp.push(proc_info)
            }
        }
        temp
    };

    if proc_infos.is_empty() {
        return proc_infos;
    }

    // Sorting oldest and newest
    let proc_infos = if settings.oldest || settings.newest {
        proc_infos.sort_by(|a, b| {
            b.clone()
                .start_time()
                .unwrap()
                .cmp(&a.clone().start_time().unwrap())
        });

        let start_time = if settings.newest {
            proc_infos.first().cloned().unwrap().start_time().unwrap()
        } else {
            proc_infos.last().cloned().unwrap().start_time().unwrap()
        };

        // There might be some process start at same time, so need to be filtered.
        let mut filtered = proc_infos
            .iter()
            .filter(|it| (*it).clone().start_time().unwrap() == start_time)
            .collect::<Vec<_>>();

        if settings.newest {
            filtered.sort_by(|a, b| b.pid.cmp(&a.pid))
        } else {
            filtered.sort_by(|a, b| a.pid.cmp(&b.pid))
        }

        vec![filtered.first().cloned().unwrap().clone()]
    } else {
        proc_infos
    };

    proc_infos
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            arg!(-e --echo                      "display PIDs before waiting"),
            arg!(-c --count                     "count of matching processes"),
            arg!(-f --full                      "use full process name to match"),
            // arg!(-g --pgroup        <PGID>      "match listed process group IDs"),
            // arg!(-G --group         <GID>       "match real group IDs"),
            arg!(-i --"ignore-case"             "match case insensitively"),
            arg!(-n --newest                    "select most recently started"),
            arg!(-o --oldest                    "select least recently started"),
            arg!(-O --older         <seconds>   "select where older than seconds"),
            // arg!(-P --parent        <PPID>      "match only child processes of the given parent"),
            // arg!(-s --session       <SID>       "match session IDs"),
            arg!(-t --terminal      <tty>       "match by controlling terminal"),
            // arg!(-u --euid          <ID>        "match by effective IDs"),
            // arg!(-U --uid           <ID>        "match by real IDs"),
            arg!(-x --exact                     "match exactly with the command name"),
            // arg!(-F --pidfile       <file>      "read PIDs from file"),
            // arg!(-L --logpidfile                "fail if PID file is not locked"),
            arg!(-r --runstates     <state>     "match runstates [D,S,Z,...]"),
            // arg!(-A --"ignore-ancestors"        "exclude our ancestors from results"),
        ])
        .arg(
            Arg::new("pattern")
                .help("Name of the program to find the PID of")
                .action(ArgAction::Append)
                .index(1),
        )
}
