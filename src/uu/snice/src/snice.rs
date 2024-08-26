// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, Command};
use uu_pgrep::process::Teletype;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("snice.md");
const USAGE: &str = help_usage!("snice.md");

#[derive(Debug)]
enum ExpressionType {
    Command(String),
    Pid(usize),
    Tty(Teletype),
    User(String),
}

#[derive(Debug)]
struct Settings{
    
}


#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            // Options
            // arg!(-f --fast          "fast mode (not implemented)"),
            // arg!(-i --interactive   "interactive"),
            arg!(-l --list          "list all signal names"),
            arg!(-L --table         "list all signal names in a nice table"),
            // arg!(-n --"no-action"   "do not actually kill processes; just print what would happen"),
            arg!(-v --verbose       "explain what is being done"),
            // arg!(-w --warnings      "enable warnings (not implemented)"),
            // Expressions
            arg!(-c --command   <command>   "expression is a command name"),
            arg!(-p --pid       <pid>       "expression is a process id number"),
            arg!(-t --tty       <tty>       "expression is a terminal"),
            arg!(-u --user      <username>  "expression is a username"),
        ])
}
