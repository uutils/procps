// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, Command};
use uu_pgrep::process_matcher;
use uucore::error::UResult;
use wait::wait;

mod wait;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = process_matcher::get_match_settings(&matches)?;
    let mut proc_infos = process_matcher::find_matching_pids(&settings)?;

    // For empty result
    if proc_infos.is_empty() {
        uucore::error::set_exit_code(1);
    }

    // Process outputs
    if matches.get_flag("count") {
        println!("{}", proc_infos.len());
    }

    if matches.get_flag("echo") {
        if settings.newest || settings.oldest {
            for ele in &proc_infos {
                println!("waiting for  (pid {})", ele.pid);
            }
        } else {
            for ele in proc_infos.iter_mut() {
                println!("waiting for {} (pid {})", ele.name().unwrap(), ele.pid);
            }
        }
    }

    wait(&proc_infos);

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Wait for processes based on name")
        .override_usage("pidwait [options] pattern")
        .infer_long_args(true)
        .args([arg!(-e --echo                      "display PIDs before waiting")])
        .args(process_matcher::clap_args(
            "Name of the program to wait for",
            true,
        ))
}
