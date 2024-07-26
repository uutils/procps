// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod collector;

#[cfg(target_os = "linux")]
use clap::crate_version;
use clap::{Arg, ArgAction, Command};
use std::{cell::RefCell, rc::Rc};
use uu_pgrep::process::walk_process;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("ps.md");
const USAGE: &str = help_usage!("ps.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let snapshot = walk_process()
        .map(|it| Rc::new(RefCell::new(it)))
        .collect::<Vec<_>>();
    let mut proc_infos = Vec::new();

    proc_infos.extend(collector::process_collector(&matches, snapshot.clone()));
    proc_infos.extend(collector::session_collector(&matches, snapshot.clone()));
    proc_infos.extend(collector::terminal_collector(&matches, snapshot));

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .arg(Arg::new("help").long("help").action(ArgAction::Help))
        .args([
            Arg::new("A")
                .short('A')
                .help("all processes")
                .visible_alias("e")
                .action(ArgAction::SetTrue),
            Arg::new("a")
                .short('a')
                .help("all with tty, except session leaders")
                .action(ArgAction::SetTrue),
            // Arg::new("a_")
            //     .short('a')
            //     .help("all with tty, including other users")
            //     .action(ArgAction::SetTrue)
            //     .allow_hyphen_values(true),
            Arg::new("d")
                .short('d')
                .help("all except session leaders")
                .action(ArgAction::SetTrue),
            Arg::new("deselect")
                .long("deselect")
                .short('N')
                .help("negate selection")
                .action(ArgAction::SetTrue),
            // Arg::new("r")
            //     .short('r')
            //     .action(ArgAction::SetTrue)
            //     .help("only running processes")
            //     .allow_hyphen_values(true),
            // Arg::new("T")
            //     .short('T')
            //     .action(ArgAction::SetTrue)
            //     .help("all processes on this terminal")
            //     .allow_hyphen_values(true),
            // Arg::new("x")
            //     .short('x')
            //     .action(ArgAction::SetTrue)
            //     .help("processes without controlling ttys")
            //     .allow_hyphen_values(true),
        ])
    // .args([
    //     Arg::new("command").short('c').help("command name"),
    //     Arg::new("GID")
    //         .short('G')
    //         .long("Group")
    //         .help("real group id or name"),
    //     Arg::new("group")
    //         .short('g')
    //         .long("group")
    //         .help("session or effective group name"),
    //     Arg::new("PID").short('p').long("pid").help("process id"),
    //     Arg::new("pPID").long("ppid").help("parent process id"),
    //     Arg::new("qPID")
    //         .short('q')
    //         .long("quick-pid")
    //         .help("process id"),
    //     Arg::new("session")
    //         .short('s')
    //         .long("sid")
    //         .help("session id"),
    //     Arg::new("t").short('t').long("tty").help("terminal"),
    //     Arg::new("eUID")
    //         .short('u')
    //         .long("user")
    //         .help("effective user id or name"),
    //     Arg::new("rUID")
    //         .short('U')
    //         .long("User")
    //         .help("real user id or name"),
    // ])
}
