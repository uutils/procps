// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fs;
use std::path::PathBuf;

use clap::{crate_version, Arg, ArgAction, ArgMatches, Command};
#[cfg(unix)]
use rustix::process::geteuid;
use uu_pgrep::process::{walk_process, ProcessInformation};
use uucore::error::UResult;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let arg_program_name = matches.get_many::<String>("program-name");
    let arg_separator = matches.get_one::<String>("S").unwrap();

    if arg_program_name.is_none() {
        uucore::error::set_exit_code(1);
        return Ok(());
    };

    let collected = collect_matched_pids(&matches);

    if collected.is_empty() {
        uucore::error::set_exit_code(1);
        return Ok(());
    };

    let output = collected
        .into_iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>()
        .join(arg_separator);

    let flag_quiet = matches.get_flag("q");
    if !flag_quiet {
        println!("{output}");
    }

    Ok(())
}

fn match_process_name(
    process: &mut ProcessInformation,
    name_to_match: &str,
    with_workers: bool,
    match_scripts: bool,
) -> bool {
    // `cmdline` has the arguments joined with spaces, so read them again to
    // get argv[0] as it is.
    let Ok(raw) = fs::read(format!("/proc/{}/cmdline", process.pid)) else {
        return false;
    };
    let binding = raw
        .split(|&b| b == 0)
        .map(String::from_utf8_lossy)
        .collect::<Vec<_>>();
    let path = binding.first().map(|it| it.to_string()).unwrap_or_default();

    if path.is_empty() {
        if !with_workers {
            return false;
        }
        return process.name().unwrap() == name_to_match;
    };

    if PathBuf::from(&path).file_name().and_then(|f| f.to_str()) == Some(name_to_match) {
        return true;
    }

    // Like procps-ng, also match the executable, so a process started with
    // another argv[0] (`exec -a`) is still found, also by the 15-byte name
    // the kernel cuts it to.
    if let Ok(exe) = fs::read_link(format!("/proc/{}/exe", process.pid)) {
        if let Some(exe) = exe.file_name().and_then(|f| f.to_str()) {
            if exe == name_to_match || exe.as_bytes().get(..15) == Some(name_to_match.as_bytes()) {
                return true;
            }
        }
    }

    // A process that rewrites its title, like "sshd: /usr/sbin/sshd -D", is
    // matched by its process name too.
    if path.contains(' ') && process.name().unwrap() == name_to_match {
        return true;
    }

    // When a script (ie. file starting with e.g. #!/bin/sh) is run like `./script.sh`, then
    // its cmdline will look like `/bin/sh ./script.sh` but its .name() will be `script.sh`.
    // As name() gets truncated to 15 characters, the original pidof seems to always do a prefix match.
    if match_scripts && binding.len() > 1 {
        return PathBuf::from(binding[1].as_ref())
            .file_name()
            .map(|f| f.to_str().unwrap())
            .is_some_and(|f| f == name_to_match && f.starts_with(&process.name().unwrap()));
    }

    false
}

fn collect_matched_pids(matches: &ArgMatches) -> Vec<usize> {
    let program_names: Vec<_> = matches
        .get_many::<String>("program-name")
        .unwrap()
        .cloned()
        .collect();
    let with_workers = matches.get_flag("with-workers");
    let match_scripts = matches.get_flag("x");

    let collected = walk_process().collect::<Vec<_>>();
    let arg_omit_pid = matches
        .get_many::<usize>("o")
        .unwrap_or_default()
        .copied()
        .collect::<Vec<_>>();

    // Original pidof silently ignores the check-root option if the user is not root.
    #[cfg(unix)]
    let check_root = matches.get_flag("check-root") && geteuid().as_raw() == 0;
    #[cfg(not(unix))]
    let check_root = false;
    let our_root = ProcessInformation::current_process_info()
        .unwrap()
        .root()
        .unwrap();

    program_names
        .into_iter()
        .flat_map(|program| {
            let mut processed = Vec::new();
            for mut process in collected.clone() {
                if !match_process_name(&mut process, &program, with_workers, match_scripts) {
                    continue;
                }
                if arg_omit_pid.contains(&process.pid) {
                    continue;
                }
                if check_root && process.root().unwrap() != our_root {
                    continue;
                }

                if matches.get_flag("t") {
                    processed.extend_from_slice(process.thread_ids());
                } else {
                    processed.push(process.pid);
                }
            }

            processed.sort_by(|a, b| b.cmp(a));

            let flag_s = matches.get_flag("s");
            if flag_s {
                match processed.first() {
                    Some(first) => vec![*first],
                    None => Vec::new(),
                }
            } else {
                processed
            }
        })
        .collect()
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Find the process ID of a running program")
        .override_usage("pidof [options] [program [...]]")
        .infer_long_args(true)
        .arg(
            Arg::new("program-name")
                .help("Program name.")
                .index(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("check-root")
                .short('c')
                .long("check-root")
                .help("Only return PIDs with the same root directory")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("S")
                .short('S')
                // the pidof bundled with Debian uses -d instead of -S
                .visible_short_alias('d')
                .long("separator")
                .help("Use SEP as separator between PIDs")
                .action(ArgAction::Set)
                .value_name("SEP")
                .default_value(" ")
                .hide_default_value(true),
        )
        // .arg(
        //     Arg::new("n")
        //         .short('n')
        //         .help("Avoid using stat system function on network shares")
        //         .action(ArgAction::SetTrue),
        // )
        .arg(
            Arg::new("o")
                .short('o')
                .long("omit-pid")
                .help("Omit results with a given PID")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(usize))
                .value_name("PID"),
        )
        .arg(
            Arg::new("q")
                .short('q')
                .help("Quiet mode. Do not display output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("s")
                .short('s')
                .long("single-shot")
                .help("Only return one PID")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("t")
                .short('t')
                .long("lightweight")
                .help("Show thread ids instead of process ids")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("with-workers")
                .short('w')
                .long("with-workers")
                .help("Show kernel worker threads as well")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("x")
                .short('x')
                .help("Return PIDs of shells running scripts with a matching name")
                .action(ArgAction::SetTrue),
        )
}
