use clap::{arg, crate_version, Arg, ArgAction, Command};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("pidof.md");
const USAGE: &str = help_usage!("pidof.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    todo!()
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(Arg::new("program-name"))
        .args([
            arg!(-c         "Return PIDs with the same root directory").action(ArgAction::SetTrue),
            arg!(-d <sep>   "Use the provided character as output separator"),
            arg!(-h         "Display this help text").action(ArgAction::Help),
            arg!(-n         "Avoid using stat system function on network shares")
                .action(ArgAction::SetTrue),
            arg!(-o <pid>   "Omit results with a given PID").action(ArgAction::Set),
            arg!(-q         "Quiet mode. Do not display output").action(ArgAction::SetTrue),
            arg!(-s         "Only return one PID").action(ArgAction::SetTrue),
            arg!(-x         "Return PIDs of shells running scripts with a matching name")
                .action(ArgAction::SetTrue),
            arg!(-z         "List zombie and I/O waiting processes. May cause pidof to hang.")
                .action(ArgAction::SetTrue),
        ])
}
