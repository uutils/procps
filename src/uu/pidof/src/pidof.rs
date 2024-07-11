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
    let arg_separator = matches.get_one::<String>("d").unwrap();

    if arg_program_name.is_none() {
        uucore::error::set_exit_code(1);
        return Ok(());
    };

    let mut collected = collect_matched_pids(&matches);

    if collected.is_empty() {
        uucore::error::set_exit_code(1);
        return Ok(());
    };

    collected.sort_by(|a, b| b.pid.cmp(&a.pid));

    let output = collected
        .into_iter()
        .map(|it| it.pid.to_string())
        .collect::<Vec<_>>()
        .join(arg_separator);

    let flag_quite = matches.get_flag("q");
    if !flag_quite {
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

fn collect_matched_pids(matches: &ArgMatches) -> Vec<ProcessInformation> {
    let program_name: Vec<_> = matches
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

    let mut processed = Vec::new();
    for mut process in collected {
        let contains = program_name.contains(&get_executable_name(&mut process));
        let should_omit = arg_omit_pid.contains(&process.pid);

        if contains && !should_omit {
            processed.push(process)
        }
    }

    let flag_s = matches.get_flag("s");
    if flag_s {
        match processed.first() {
            Some(first) => vec![first.clone()],
            None => Vec::new(),
        }
    } else {
        processed
    }
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
                .required(true)
                .index(1),
        )
        // .arg(
        //     Arg::new("c")
        //         .short('c')
        //         .help("Return PIDs with the same root directory")
        //         .action(ArgAction::SetTrue),
        // )
        .arg(
            Arg::new("d")
                .short('d')
                .help("Use the provided character as output separator")
                .action(ArgAction::Set)
                .value_name("sep")
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
                .help("Omit results with a given PID")
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(usize))
                .value_name("omitpid"),
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
                .help("Only return one PID")
                .action(ArgAction::SetTrue),
        )
    // .arg(
    //     Arg::new("x")
    //         .short('x')
    //         .help("Return PIDs of shells running scripts with a matching name")
    //         .action(ArgAction::SetTrue),
    // )
}
