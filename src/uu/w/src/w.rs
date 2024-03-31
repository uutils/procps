// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::crate_version;
use clap::{Arg, ArgAction, Command};
use std::process;
#[cfg(not(windows))]
use uucore::utmpx::Utmpx;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("w.md");
const USAGE: &str = help_usage!("w.md");

struct UserInfo {
    user: String,
    terminal: String,
    login_time: String,
    idle_time: String,
    jcpu: String,
    pcpu: String,
    command: String,
}

#[cfg(not(windows))]
fn fetch_user_info() -> Result<Vec<UserInfo>, std::io::Error> {
    let mut user_info_list = Vec::new();
    for entry in Utmpx::iter_all_records() {
        if entry.is_user_process() {
            let user_info = UserInfo {
                user: entry.user(),
                terminal: entry.tty_device(),
                login_time: format!("{}", entry.login_time()), // Needs formatting
                idle_time: String::new(), // Placeholder, needs actual implementation
                jcpu: String::new(),      // Placeholder, needs actual implementation
                pcpu: String::new(),      // Placeholder, needs actual implementation
                command: String::new(),   // Placeholder, needs actual implementation
            };
            user_info_list.push(user_info);
        }
    }

    Ok(user_info_list)
}

#[cfg(windows)]
fn fetch_user_info() -> Result<Vec<UserInfo>, std::io::Error> {
    Ok(Vec::new())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let no_header = matches.get_flag("no-header");

    match fetch_user_info() {
        Ok(user_info) => {
            if !no_header {
                println!("USER\tTTY\t\tLOGIN@\t\tIDLE\tJCPU\tPCPU\tWHAT");
            }
            for user in user_info {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    user.user,
                    user.terminal,
                    user.login_time,
                    user.idle_time,
                    user.jcpu,
                    user.pcpu,
                    user.command
                );
            }
        }
        Err(e) => {
            eprintln!("w: failed to fetch user info: {}", e);
            process::exit(1);
        }
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
        .arg(
            Arg::new("no-header")
                .short('h')
                .long("no-header")
                .help("do not print header")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-current")
                .short('u')
                .long("no-current")
                .help("ignore current process username")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("short")
                .short('s')
                .long("short")
                .help("short format")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("from")
                .short('f')
                .long("from")
                .help("show remote hostname field")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("old-style")
                .short('o')
                .long("old-style")
                .help("old style output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ip-addr")
                .short('i')
                .long("ip-addr")
                .help("display IP address instead of hostname (if possible)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pids")
                .short('p')
                .long("pids")
                .help("show the PID(s) of processes in WHAT")
                .action(ArgAction::SetTrue),
        )
}
