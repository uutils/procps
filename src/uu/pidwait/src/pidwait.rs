// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, ArgMatches, Command};
use regex::Regex;
use std::{collections::HashSet, env, sync::OnceLock};
use uu_pgrep::process::{walk_process, ProcessInformation, RunState, Teletype};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

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

    if !settings.newest
        && !settings.oldest
        && settings.runstates.is_none()
        && settings.older.is_none()
        && settings.terminal.is_none()
    {
        return Err(USimpleError::new(
            2,
            "no matching criteria specified\nTry `pidwait --help' for more information.",
        ));
    }

    let pattern = try_get_pattern_from(&matches, &settings)?;
    REGEX
        .set(Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?)
        .unwrap();

    let mut proc_infos = collect_proc_infos(&settings);

    // Process outputting
    if settings.count {
        println!("{}", proc_infos.len())
    }

    if settings.echo {
        for ele in proc_infos.iter_mut() {
            println!("waiting for {} (pid {})", ele.status()["Name"], ele.pid)
        }
    }

    Ok(())
}

fn try_get_pattern_from(matches: &ArgMatches, settings: &Settings) -> UResult<String> {
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

    Ok(pattern.to_string())
}

fn collect_proc_infos(settings: &Settings) -> Vec<ProcessInformation> {
    let proc_infos: Vec<_> = walk_process().collect();
    if settings.oldest || settings.newest {
        if settings.oldest {
        } else if settings.newest {
        }
    }

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
}
