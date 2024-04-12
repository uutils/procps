// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, ArgAction, Command};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("slabtop.md");
const USAGE: &str = help_usage!("slabtop.md");

mod parse;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let sort_flag = matches
        .try_get_one::<char>("sort")
        .ok()
        .unwrap_or(Some(&'o'))
        .unwrap_or(&'o');

    // TODO: LISTEN TO REFRESH
    loop {
        parse::SlabInfo::new()?.sort(*sort_flag, true);

        // TODO: EXIT
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .args([
            // arg!(-d --delay <secs>  "delay updates"),
            // arg!(-o --once          "only display once, then exit"),
            arg!(-s --sort  <char>  "specify sort criteria by character (see below)"),
            arg!(-h --help          "display this help and exit").action(ArgAction::Help),
        ])
        .after_help(
            r"The following are valid sort criteria:
 a: sort by number of active objects
 b: sort by objects per slab
 c: sort by cache size
 l: sort by number of slabs
 v: sort by (non display) number of active slabs
 n: sort by name
 o: sort by number of objects (the default)
 p: sort by (non display) pages per slab
 s: sort by object size
 u: sort by cache utilization",
        )
}
