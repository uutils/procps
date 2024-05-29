// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod pid;

use clap::{arg, crate_version, Arg, ArgAction, Command};
use pid::walk_pid;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let default_delimiter = "\n".into();
    let delimiter = matches
        .get_one::<String>("delimiter")
        .unwrap_or(&default_delimiter);

    let mut programs = matches
        .get_many::<String>("pattern")
        .unwrap_or_default()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let should_ignore_case = matches.get_flag("ignore-case");
    let should_inverse = matches.get_flag("inverse");

    let flag_list_name = matches.get_flag("list-name");
    let flag_list_full = matches.get_flag("list-full");

    let flag_count = matches.get_flag("count");
    // let flag_full = matches.get_flag("full");

    if should_ignore_case {
        programs = programs.into_iter().map(|it| it.to_lowercase()).collect();
    }

    let result: Vec<_> = walk_pid()
        .filter(|it| {
            let Some(name) = it.status.get("Name") else {
                return false;
            };

            // Processs flag `--ignore-case`
            let name = if should_ignore_case {
                name.to_lowercase()
            } else {
                name.into()
            };

            programs.iter().any(|it| name.contains(it)) ^ should_inverse
        })
        .collect();

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
                    let name = it.status.get("Name").cloned().unwrap_or_default();
                    format!("{} {}", it.pid, name)
                } else {
                    format!("{}", it.pid)
                }
            })
            .collect::<Vec<_>>();
        println!("{}", result.join(delimiter));
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
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
