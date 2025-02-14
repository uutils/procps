// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
pub mod process;
pub mod process_matcher;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, Command};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

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
    let settings = process_matcher::get_match_settings(&matches)?;

    // Collect pids
    let pids = process_matcher::find_matching_pids(&settings);

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
            arg!(-H     --"require-handler"     "match only if signal handler is present"),
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
            arg!(--signal <sig>                 "signal to send (either number or name)")
                .default_value("SIGTERM"),
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
