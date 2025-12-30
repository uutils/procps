// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use std::env;
use sysinfo::{Pid, System};
use uucore::error::{set_exit_code, UResult, USimpleError};

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let pids = matches.get_many::<String>("pid").unwrap();
    let sys = System::new_all();

    for pid_str in pids {
        let pid = match pid_str.parse::<usize>() {
            // PIDs start at 1, hence 0 is invalid
            Ok(0) | Err(_) => {
                return Err(USimpleError::new(
                    1,
                    format!("invalid process id: {pid_str}"),
                ))
            }
            Ok(pid) => pid,
        };

        match sys.process(Pid::from(pid)) {
            Some(process) => match process.cwd() {
                Some(cwd) => println!("{pid}: {}", cwd.display()),
                None => {
                    set_exit_code(1);
                    eprintln!("{pid}: Permission denied");
                }
            },
            None => {
                set_exit_code(1);
                eprintln!("{pid}: No such process");
            }
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Report current working directory of a process")
        .override_usage("pwdx [options] pid [...]")
        .infer_long_args(true)
        .arg(
            Arg::new("pid")
                .value_name("PID")
                .help("Process ID")
                .required(true)
                .num_args(1..)
                .index(1),
        )
}
