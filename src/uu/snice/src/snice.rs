// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{collections::HashSet, path::PathBuf, str::FromStr};

use crate::priority::Priority;
pub use action::ActionResult;
use action::{perform_action, process_snapshot, users, SelectedTarget};
use clap::{crate_version, Arg, Command};
use prettytable::{format::consts::FORMAT_CLEAN, row, Table};
pub use process_matcher::clap_args;
use process_matcher::*;
use sysinfo::Pid;
use uu_pgrep::process::ProcessInformation;
#[cfg(target_family = "unix")]
use uucore::signals::ALL_SIGNALS;
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("snice.md");
const USAGE: &str = help_usage!("snice.md");

mod action;
mod priority;
pub mod process_matcher;

#[derive(Debug)]
pub enum SignalDisplay {
    List,
    Table,
}

#[allow(unused)]
impl SignalDisplay {
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

#[allow(unused)] // unused argument under non-unix targets
pub fn print_signals(display: &SignalDisplay) {
    #[cfg(target_family = "unix")]
    {
        let result = display.display(&ALL_SIGNALS);

        println!("{result}");
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let settings = Settings::try_new(&matches)?;

    // Case0: Print SIGNALS
    if let Some(display) = &settings.display {
        print_signals(display);
        return Ok(());
    }

    // Case1: Perform priority
    let take_action = !matches.get_flag("no-action");
    if let Some(targets) = settings.expressions {
        let priority_str = matches.get_one::<String>("priority").cloned();

        let priority = match priority_str {
            Some(expr) => {
                Priority::try_from(expr).map_err(|err| USimpleError::new(1, err.to_string()))?
            }
            None => Priority::default(),
        };

        let pids = collect_pids(&targets);
        let results = perform_action(&pids, &priority, take_action);

        if results.iter().all(|it| it.is_none()) || results.is_empty() {
            return Err(USimpleError::new(1, "no process selection criteria"));
        }

        if settings.verbose {
            let output = construct_verbose_result(&pids, &results).trim().to_owned();
            println!("{output}");
        } else if !take_action {
            pids.iter().for_each(|pid| println!("{pid}"));
        }
    }

    Ok(())
}

#[allow(unused)]
pub fn construct_verbose_result(pids: &[u32], action_results: &[Option<ActionResult>]) -> String {
    let mut table = action_results
        .iter()
        .enumerate()
        .map(|(index, it)| (pids[index], it))
        .filter(|(_, it)| it.is_some())
        .map(|(pid, action)| (pid, action.clone().unwrap()))
        .map(|(pid, action)| {
            let process = process_snapshot().process(Pid::from_u32(pid)).unwrap();

            let tty =
                ProcessInformation::try_new(PathBuf::from_str(&format!("/proc/{pid}")).unwrap());

            let user = process
                .user_id()
                .and_then(|uid| users().iter().find(|it| it.id() == uid))
                .map(|it| it.name())
                .unwrap_or("?")
                .to_owned();

            let mut cmd = process
                .exe()
                .and_then(|it| it.iter().next_back())
                .unwrap_or("?".as_ref());
            let cmd = cmd.to_str().unwrap();

            (tty, user, pid, cmd, action)
        })
        .filter(|(tty, _, _, _, _)| tty.is_ok())
        .map(|(tty, user, pid, cmd, action)| row![tty.unwrap().tty(), user, pid, cmd, action])
        .collect::<Table>();

    table.set_format(*FORMAT_CLEAN);

    table.to_string()
}

/// Map and sort `SelectedTarget` to pids.
pub fn collect_pids(targets: &[SelectedTarget]) -> Vec<u32> {
    let collected = targets
        .iter()
        .flat_map(SelectedTarget::to_pids)
        .collect::<HashSet<_>>();

    let mut collected = collected.into_iter().collect::<Vec<_>>();
    collected.sort_unstable();
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
        .args(clap_args())
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
