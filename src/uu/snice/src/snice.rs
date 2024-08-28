// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::collections::HashSet;

use action::{perform_action, SelectedTarget};
use clap::{arg, crate_version, value_parser, Arg, ArgMatches, Command};
use priority::Priority;
use uu_pgrep::process::Teletype;
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
    signals::ALL_SIGNALS,
};

const ABOUT: &str = help_about!("snice.md");
const USAGE: &str = help_usage!("snice.md");

mod action;
mod priority;

#[derive(Debug)]
enum SignalDisplay {
    List,
    Table,
}

impl SignalDisplay {
    fn try_new(matches: &ArgMatches) -> Option<SignalDisplay> {
        if matches.get_flag("table") {
            Some(SignalDisplay::Table)
        } else if matches.get_flag("list") {
            Some(SignalDisplay::List)
        } else {
            None
        }
    }

    fn display(&self, signals: &[&str]) -> String {
        match self {
            SignalDisplay::List => Self::list(signals),
            SignalDisplay::Table => Self::table(signals),
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
    expressions: Option<Vec<SelectedTarget>>,
    priority: Priority,
}

impl Settings {
    fn try_new(matches: &ArgMatches) -> UResult<Self> {
        let priority = matches
            .try_get_one::<String>("priority")
            .unwrap_or(Some(&"".to_string()))
            .cloned();

        let expression = match priority {
            Some(expr) => {
                Priority::try_from(expr).map_err(|err| USimpleError::new(1, err.to_string()))?
            }
            None => Priority::default(),
        };

        Ok(Self {
            display: SignalDisplay::try_new(matches),
            expressions: Self::targets(matches),
            priority: expression,
        })
    }

    fn targets(matches: &ArgMatches) -> Option<Vec<SelectedTarget>> {
        let cmd = matches
            .get_many::<String>("command")
            .unwrap_or_default()
            .map(Into::into)
            .map(SelectedTarget::Command)
            .collect::<Vec<_>>();

        let pid = matches
            .get_many::<u32>("pid")
            .unwrap_or_default()
            .map(Clone::clone)
            .map(SelectedTarget::Pid)
            .collect::<Vec<_>>();

        let tty = matches
            .get_many::<String>("tty")
            .unwrap_or_default()
            .flat_map(|it| Teletype::try_from(it.as_str()))
            .map(SelectedTarget::Tty)
            .collect::<Vec<_>>();

        let user = matches
            .get_many::<String>("user")
            .unwrap_or_default()
            .map(Into::into)
            .map(SelectedTarget::User)
            .collect::<Vec<_>>();

        let collected = cmd
            .into_iter()
            .chain(pid)
            .chain(tty)
            .chain(user)
            .collect::<Vec<_>>();

        if collected.is_empty() {
            None
        } else {
            Some(collected)
        }
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = Settings::try_new(&matches)?;

    // Case0: Print SIGNALS
    if let Some(display) = settings.display {
        let result = display.display(&ALL_SIGNALS);
        println!("{result}");
        return Ok(());
    }

    // Case1: Perform priority
    if let Some(targets) = settings.expressions {
        let pids = collect_pids(&targets);
        perform_action(&pids, &settings.priority);
    }

    Ok(())
}

/// Map and sort `SelectedTarget` to pids.
fn collect_pids(targets: &[SelectedTarget]) -> Vec<u32> {
    let collected = targets
        .iter()
        .flat_map(SelectedTarget::to_pids)
        .collect::<HashSet<_>>();

    let mut collected = collected.into_iter().collect::<Vec<_>>();
    collected.sort();
    collected
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg_required_else_help(true)
        .arg(Arg::new("priority"))
        .args([
            // Options
            // arg!(-f --fast          "fast mode (not implemented)"),
            // arg!(-i --interactive   "interactive"),
            arg!(-l --list                  "list all signal names"),
            arg!(-L --table                 "list all signal names in a nice table"),
            // arg!(-n --"no-action"   "do not actually kill processes; just print what would happen"),
            // arg!(-v --verbose               "explain what is being done"),
            // arg!(-w --warnings      "enable warnings (not implemented)"),
            // Expressions
            arg!(-c --command   <command>   ...   "expression is a command name"),
            arg!(-p --pid       <pid>       ...   "expression is a process id number")
                .value_parser(value_parser!(u32)),
            arg!(-t --tty       <tty>       ...   "expression is a terminal"),
            arg!(-u --user      <username>  ...   "expression is a username"),
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
