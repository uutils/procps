use clap::{crate_version, Arg, ArgAction, Command};
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
        .arg(
            Arg::new("program")
                .help("Program name.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("c")
                .short('c')
                .help("Return PIDs with the same root directory")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("d")
                .short('d')
                .help("Use the provided character as output separator")
                .action(ArgAction::Set)
                .value_name("sep")
                .default_value(" ")
                .hide_default_value(true),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .help("Display this help text")
                .action(ArgAction::Help),
        )
        .arg(
            Arg::new("n")
                .short('n')
                .help("Avoid using stat system function on network shares")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("o")
                .short('o')
                .help("Omit results with a given PID")
                .action(ArgAction::Set)
                .value_name("pid"),
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
        .arg(
            Arg::new("x")
                .short('x')
                .help("Return PIDs of shells running scripts with a matching name")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("z")
                .short('z')
                .help("List zombie and I/O waiting processes. May cause pidof to hang.")
                .action(ArgAction::SetTrue),
        )
}
