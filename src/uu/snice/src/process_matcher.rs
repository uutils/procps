// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::action::SelectedTarget;
use crate::SignalDisplay;
use clap::{arg, value_parser, Arg, ArgMatches};
use uu_pgrep::process::Teletype;
use uucore::error::UResult;

#[derive(Debug)]
pub struct Settings {
    pub display: Option<SignalDisplay>,
    pub expressions: Option<Vec<SelectedTarget>>,
    pub verbose: bool,
    pub warnings: bool,
    pub no_action: bool,
}

impl Settings {
    pub fn try_new(matches: &ArgMatches) -> UResult<Self> {
        let display = if matches.get_flag("table") {
            Some(SignalDisplay::Table)
        } else if matches.get_flag("list") {
            Some(SignalDisplay::List)
        } else {
            None
        };

        Ok(Self {
            display,
            expressions: Self::targets(matches),
            verbose: matches.get_flag("verbose"),
            warnings: matches.get_flag("warnings"),
            no_action: matches.get_flag("no-action"),
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

#[allow(clippy::cognitive_complexity)]
pub fn clap_args() -> Vec<Arg> {
    vec![
        // arg!(-f --fast          "fast mode (not implemented)"),
        // arg!(-i --interactive   "interactive"),
        arg!(-l --list                  "list all signal names"),
        arg!(-L --table                 "list all signal names in a nice table"),
        arg!(-n --"no-action"   "do not actually kill processes; just print what would happen"),
        arg!(-v --verbose               "explain what is being done"),
        arg!(-w --warnings      "enable warnings (not implemented)"),
        // Expressions
        arg!(-c --command   <command>   ...   "expression is a command name"),
        arg!(-p --pid       <pid>       ...   "expression is a process id number")
            .value_parser(value_parser!(u32)),
        arg!(-t --tty       <tty>       ...   "expression is a terminal"),
        arg!(-u --user      <username>  ...   "expression is a username"),
        // arg!(--ns <PID>                "match the processes that belong to the same namespace as <pid>"),
        // arg!(--nslist <ns>             "list which namespaces will be considered for the --ns option.")
        //     .value_delimiter(',')
        //     .value_parser(["ipc", "mnt", "net", "pid", "user", "uts"]),
    ]
}
