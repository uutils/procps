// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path::PathBuf;

use clap::{crate_version, Arg, ArgAction, ArgMatches, Command};
use uu_pgrep::process::{walk_process, ProcessInformation};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pidof.md");
const USAGE: &str = help_usage!("pidof.md");

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

fn get_executable_name(process: &mut ProcessInformation) -> String {
    let binding = process.cmdline.split(' ').collect::<Vec<_>>();
    let mut path = binding.first().unwrap().to_string();

    if path.is_empty() {
        path.clone_from(&process.status()["Name"]);
    };

    PathBuf::from(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn collect_matched_pids(matches: &ArgMatches) -> Vec<usize> {
    let program_names: Vec<_> = matches
        .get_many::<String>("program-name")
        .unwrap()
        .cloned()
        .collect();

    let collected = walk_process().collect::<Vec<_>>();
    let arg_omit_pid = matches
        .get_many::<usize>("o")
        .unwrap_or_default()
        .copied()
        .collect::<Vec<_>>();

    program_names
        .into_iter()
        .flat_map(|program| {
            let mut processed = Vec::new();
            for mut process in collected.clone() {
                let contains = program == get_executable_name(&mut process);
                let should_omit = arg_omit_pid.contains(&process.pid);

                if contains && !should_omit {
                    if matches.get_flag("t") {
                        processed.extend_from_slice(&process.thread_ids());
                    } else {
                        processed.push(process.pid);
                    }
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
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new("program-name")
                .help("Program name.")
                .index(1)
                .action(ArgAction::Append),
        )
        // .arg(
        //     Arg::new("c")
        //         .short('c')
        //         .help("Return PIDs with the same root directory")
        //         .action(ArgAction::SetTrue),
        // )
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
    // .arg(
    //     Arg::new("x")
    //         .short('x')
    //         .help("Return PIDs of shells running scripts with a matching name")
    //         .action(ArgAction::SetTrue),
    // )
}
