// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Pid utils
pub mod process;
pub mod process_matcher;

use clap::{arg, crate_version, Command};
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
    let mut settings = process_matcher::get_match_settings(&matches)?;
    settings.threads = matches.get_flag("lightweight");

    // Collect pids
    let pids = process_matcher::find_matching_pids(&settings)?;

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
                        format!("{} [{}]", it.pid, it.clone().name().unwrap())
                    } else {
                        format!("{} {}", it.pid, it.cmdline)
                    }
                })
                .collect()
        } else if matches.get_flag("list-name") {
            pids.into_iter()
                .map(|it| format!("{} {}", it.pid, it.clone().name().unwrap()))
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
        .args([
            arg!(-d     --delimiter <string>    "specify output delimiter")
                .default_value("\n")
                .hide_default_value(true),
            arg!(-l     --"list-name"           "list PID and process name"),
            arg!(-a     --"list-full"           "list PID and full command line"),
            arg!(-w     --lightweight           "list all TID"),
        ])
        .args(process_matcher::clap_args(
            "Name of the program to find the PID of",
            true,
        ))
}
