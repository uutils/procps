// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod command;
mod util;

use crate::command::{parse_command, Expr, Settings};
use clap::{crate_version, Arg, ArgAction, Command};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use uu_pgrep::process::ProcessInformation;
use uucore::signals::ALL_SIGNALS;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("skill.md");
const USAGE: &str = help_usage!("skill.md");
const SIGNALS_PER_ROW: usize = 7; // Be consistent with procps-ng

#[uucore::main]
pub fn uumain(mut args: impl uucore::Args) -> UResult<()> {
    let new = parse_command(&mut args);
    let matches = uu_app().try_get_matches_from(new)?;
    let mut cli = Settings::new(matches);

    // If list or table is specified, print the list of signals and return
    if cli.list || cli.table {
        list_signals(&cli);
        return Ok(());
    }

    if cli.fast {
        //TODO: implement this option
    }

    let signal = parse_signal_str(&cli.signal);

    // parse the expression if not specify its type
    parse_expression(&mut cli);

    let matching_processes = find_matching_processes(&cli.expression);

    if matching_processes.is_empty() {
        eprintln!("No matching processes found");
        return Ok(());
    }

    if cli.verbose || cli.no_action {
        for process in &matching_processes {
            println!(
                "Would send signal {} to process {} with cmd {}",
                &cli.signal, process.pid, process.cmdline
            );
        }
        if cli.no_action {
            return Ok(());
        }
    }

    if cli.interactive {
        for mut process in matching_processes {
            let cmd = process.cmdline.clone();
            let owner =
                util::get_process_owner(&mut process).unwrap_or_else(|| "<unknown>".to_string());
            let tty =
                util::get_process_terminal(&process).unwrap_or_else(|| "<unknown>".to_string());
            if confirm_action(&tty, &owner, process.pid as i32, &cmd) {
                if let Err(e) = send_signal(process.pid as i32, signal) {
                    if cli.warnings {
                        eprintln!("Failed to send signal to process {}: {}", process.pid, e);
                    }
                }
            } else {
                println!("Skipping process {}", process.pid);
            }
        }
    } else {
        for process in matching_processes {
            if let Err(e) = send_signal(process.pid as i32, signal) {
                if cli.warnings {
                    eprintln!("Failed to send signal to process {}: {}", process.pid, e);
                }
            }
        }
    }

    Ok(())
}

// TODO: add more strict check according to the usage
fn parse_expression(cli: &mut Settings) {
    if let Expr::Raw(raw_expr) = &cli.expression {
        // Check if any strings in the raw expression match active users, commands, or terminals
        if raw_expr.iter().all(|s| s.parse::<i32>().is_ok()) {
            cli.expression =
                Expr::Pid(raw_expr.iter().map(|s| s.parse::<i32>().unwrap()).collect());
        } else {
            let mut processes = util::get_all_processes();
            let is_user_expr = raw_expr
                .iter()
                .any(|s| util::get_active_users(&mut processes).contains(s));
            let is_command_expr = raw_expr
                .iter()
                .any(|s| util::get_active_commands(&processes).contains(s));
            let is_terminal_expr = raw_expr
                .iter()
                .any(|s| util::get_active_terminals(&processes).contains(s));
            // Only perform the replacement if we found matching users
            let raw_clone = raw_expr.clone();
            if is_user_expr {
                cli.expression = Expr::User(raw_clone);
            } else if is_command_expr {
                cli.expression = Expr::Command(raw_clone);
            } else if is_terminal_expr {
                cli.expression = Expr::Terminal(raw_clone);
            }
        }
    }
}

fn send_signal(pid: i32, signal: Signal) -> UResult<()> {
    match signal::kill(Pid::from_raw(pid), signal) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn find_matching_processes(expression: &Expr) -> Vec<ProcessInformation> {
    let mut processes = Vec::new();

    match expression {
        Expr::Pid(p) => {
            processes.extend(util::filter_processes_by_pid(p));
        }
        Expr::User(u) => {
            processes.extend(util::filter_processes_by_user(u));
        }
        Expr::Command(c) => {
            processes.extend(util::filter_processes_by_command(c));
        }
        Expr::Terminal(t) => {
            processes.extend(util::filter_processes_by_terminal(t));
        }
        _ => {
            return Vec::new();
        }
    }
    processes
}

fn confirm_action(tty: &str, owner: &str, pid: i32, cmd: &str) -> bool {
    use std::io::{stdin, stdout, Write};

    print!("{}: {} {} {}    y/N? ", tty, owner, pid, cmd);
    stdout().flush().unwrap();

    let mut input = String::new();
    if stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}

fn list_signals(cli: &Settings) {
    if cli.list {
        for signal in ALL_SIGNALS[1..].iter() {
            print!("{} ", signal);
            if signal == ALL_SIGNALS.last().unwrap() {
                println!();
            }
        }
    } else if cli.table {
        let mut result = Vec::new();
        let mut signal_num = 1;

        // Group signals into rows of 7
        for chunk in ALL_SIGNALS[1..].chunks(SIGNALS_PER_ROW) {
            let mut row = String::new();
            // Format each signal with number in the row
            for signal in chunk.iter() {
                row.push_str(&format!("{:2} {:<7}", signal_num, signal));
                signal_num += 1;
            }
            result.push(row);
        }

        for row in result {
            println!("{}", row);
        }
    }
}

fn parse_signal_str(signal: &str) -> Signal {
    match signal {
        "HUP" => Signal::SIGHUP,
        "INT" => Signal::SIGINT,
        "QUIT" => Signal::SIGQUIT,
        "ILL" => Signal::SIGILL,
        "TRAP" => Signal::SIGTRAP,
        "ABRT" => Signal::SIGABRT,
        "BUS" => Signal::SIGBUS,
        "FPE" => Signal::SIGFPE,
        "KILL" => Signal::SIGKILL,
        "USR1" => Signal::SIGUSR1,
        "SEGV" => Signal::SIGSEGV,
        "USR2" => Signal::SIGUSR2,
        "PIPE" => Signal::SIGPIPE,
        "ALRM" => Signal::SIGALRM,
        "TERM" => Signal::SIGTERM,
        "STKFLT" => Signal::SIGSTKFLT,
        "CHLD" => Signal::SIGCHLD,
        "CONT" => Signal::SIGCONT,
        "STOP" => Signal::SIGSTOP,
        "TSTP" => Signal::SIGTSTP,
        "TTIN" => Signal::SIGTTIN,
        "TTOU" => Signal::SIGTTOU,
        "URG" => Signal::SIGURG,
        "XCPU" => Signal::SIGXCPU,
        "XFSZ" => Signal::SIGXFSZ,
        "VTALRM" => Signal::SIGVTALRM,
        "PROF" => Signal::SIGPROF,
        "WINCH" => Signal::SIGWINCH,
        "POLL" => Signal::SIGIO,
        "PWR" => Signal::SIGPWR,
        "SYS" => Signal::SIGSYS,
        _ => panic!("Unknown signal: {}", signal),
    }
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("signal")
                .required(true)
                .index(1)
                .allow_hyphen_values(true)
                .default_value("TERM"),
        )
        .arg(
            Arg::new("expression")
                .help("Expression to match, can be: terminal, user, pid, command.")
                .value_name("expression")
                .required_unless_present_any(["table", "list"])
                .num_args(1..)
                .index(2),
        )
        // Flag options
        .arg(
            Arg::new("fast")
                .short('f')
                .long("fast")
                .help("fast mode (not implemented)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("interactive")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("list all signal names")
                .action(ArgAction::SetTrue)
                .conflicts_with("table"),
        )
        .arg(
            Arg::new("table")
                .short('L')
                .long("table")
                .help("list all signal names in a nice table")
                .action(ArgAction::SetTrue)
                .conflicts_with("list"),
        )
        .arg(
            Arg::new("no-action")
                .short('n')
                .long("no-action")
                .help("do not actually kill processes; just print what would happen")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("explain what is being done")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("warnings")
                .short('w')
                .long("warnings")
                .help("enable warnings (not implemented)")
                .action(ArgAction::SetTrue),
        )
        // Non-flag options
        .arg(
            Arg::new("command")
                .short('c')
                .long("command")
                .help("expression is a command name")
                .action(ArgAction::SetTrue)
                .help_heading("The options below may be used to ensure correct interpretation."),
        )
        .arg(
            Arg::new("pid")
                .short('p')
                .long("pid")
                .help("expression is a process id number")
                .action(ArgAction::SetTrue)
                .help_heading("The options below may be used to ensure correct interpretation."),
        )
        .arg(
            Arg::new("tty")
                .short('t')
                .long("tty")
                .help("expression is a terminal")
                .action(ArgAction::SetTrue)
                .help_heading("The options below may be used to ensure correct interpretation."),
        )
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .help("expression is a username")
                .action(ArgAction::SetTrue)
                .help_heading("The options below may be used to ensure correct interpretation."),
        )
}
