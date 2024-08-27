// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, ArgMatches, Command};
use uu_pgrep::process::Teletype;
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("snice.md");
const USAGE: &str = help_usage!("snice.md");

#[derive(Debug)]
enum ExpressionType {
    Command(String),
    Pid(usize),
    Tty(Teletype),
    User(String),
}

impl ExpressionType {
    fn try_new(matches: &ArgMatches) -> UResult<Option<Self>> {
        todo!()
    }
}

#[derive(Debug)]
enum SignalDisplay {
    List,
    Table,
}

impl SignalDisplay {
    fn try_new(matches: &ArgMatches) -> UResult<Option<SignalDisplay>> {
        todo!()
    }

    fn display(&self, signals: &[&str]) -> String {
        match self {
            SignalDisplay::List => Self::list(signals),
            SignalDisplay::Table => Self::list(signals),
        }
    }

    fn table(signals: &[&str]) -> String {
        let slice = &signals.to_vec()[1..];

        let formatted = slice
            .iter()
            .enumerate()
            .map(|(index, signal)| format!("{:>2} {:<8}", index + 1, signal))
            .collect::<Vec<_>>();

        formatted
            .chunks(7)
            .map(|it| it.join("").trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn list(signals: &[&str]) -> String {
        let slice = &signals.to_vec()[1..];

        slice
            .chunks(16)
            .map(|it| it.join(" "))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug)]
struct Settings {
    display: Option<SignalDisplay>,
    expression: Option<ExpressionType>,
}

impl Settings {
    fn try_new(matches: &ArgMatches) -> UResult<Self> {
        Ok(Self {
            display: SignalDisplay::try_new(matches)?,
            expression: ExpressionType::try_new(matches)?,
        })
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = Settings::try_new(&matches)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    static ALL_SIGNALS: [&str; 32] = [
        "EXIT", "HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "BUS", "FPE", "KILL", "USR1", "SEGV",
        "USR2", "PIPE", "ALRM", "TERM", "STKFLT", "CHLD", "CONT", "STOP", "TSTP", "TTIN", "TTOU",
        "URG", "XCPU", "XFSZ", "VTALRM", "PROF", "WINCH", "POLL", "PWR", "SYS",
    ];
    #[test]
    fn test_signal_display_list() {
        let output = SignalDisplay::list(&ALL_SIGNALS);

        assert_eq!(
            output,
            "HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT
CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH POLL PWR SYS"
        )
    }

    #[test]
    fn test_signal_display_table() {
        let output = SignalDisplay::table(&ALL_SIGNALS);

        assert_eq!(
            output,
            " 1 HUP      2 INT      3 QUIT     4 ILL      5 TRAP     6 ABRT     7 BUS
 8 FPE      9 KILL    10 USR1    11 SEGV    12 USR2    13 PIPE    14 ALRM
15 TERM    16 STKFLT  17 CHLD    18 CONT    19 STOP    20 TSTP    21 TTIN
22 TTOU    23 URG     24 XCPU    25 XFSZ    26 VTALRM  27 PROF    28 WINCH
29 POLL    30 PWR     31 SYS"
        )
    }
}
