// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, Command};
use std::env;
use uucore::error::UResult;
use uucore::{format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("sysctl.md");
const USAGE: &str = help_usage!("sysctl.md");

#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use uucore::error::{FromIo, UIoError};
    use walkdir::WalkDir;

    const PROC_SYS_ROOT: &str = "/proc/sys";

    pub fn get_all_sysctl_variables() -> Vec<String> {
        let mut ret = vec![];
        for entry in WalkDir::new(PROC_SYS_ROOT) {
            match entry {
                Ok(e) => {
                    if e.file_type().is_file() {
                        let var = e
                            .path()
                            .strip_prefix(PROC_SYS_ROOT)
                            .expect("Always should be ancestor of of sysctl root");
                        if let Some(s) = var.as_os_str().to_str() {
                            ret.push(s.to_owned());
                        }
                    }
                }
                Err(e) => {
                    uucore::show_error!("{}", e);
                }
            }
        }
        ret
    }

    pub fn normalize_var(var: &str) -> String {
        var.replace('/', ".")
    }

    pub fn variable_path(var: &str) -> PathBuf {
        Path::new(PROC_SYS_ROOT).join(var.replace('.', "/"))
    }

    pub fn get_sysctl(var: &str) -> std::io::Result<String> {
        Ok(std::fs::read_to_string(variable_path(var))?
            .trim_end()
            .to_string())
    }

    pub fn set_sysctl(var: &str, value: &str) -> std::io::Result<()> {
        std::fs::write(variable_path(var), value)
    }

    pub fn handle_one_arg(
        var_or_assignment: &str,
        quiet: bool,
    ) -> Result<Option<(String, String)>, Box<UIoError>> {
        let mut split = var_or_assignment.splitn(2, '=');
        let var = normalize_var(split.next().expect("Split always returns at least 1 value"));

        if let Some(value_to_set) = split.next() {
            set_sysctl(&var, value_to_set)
                .map_err(|e| e.map_err_context(|| format!("error writing key '{}'", var)))?;
            if quiet {
                Ok(None)
            } else {
                Ok(Some((var, value_to_set.to_string())))
            }
        } else {
            let value = get_sysctl(&var)
                .map_err(|e| e.map_err_context(|| format!("error reading key '{}'", var)))?;
            Ok(Some((var, value)))
        }
    }
}
#[cfg(target_os = "linux")]
use linux::*;

#[cfg(target_os = "linux")]
#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let vars = if matches.get_flag("all") {
        get_all_sysctl_variables()
    } else if let Some(vars) = matches.get_many::<String>("variables") {
        vars.cloned().collect()
    } else {
        uu_app().print_help()?;
        return Ok(());
    };

    for var_or_assignment in vars {
        match handle_one_arg(&var_or_assignment, matches.get_flag("quiet")) {
            Ok(None) => (),
            Ok(Some((var, value_to_print))) => {
                for line in value_to_print.split('\n') {
                    if matches.get_flag("names") {
                        println!("{}", var);
                    } else if matches.get_flag("values") {
                        println!("{}", line);
                    } else {
                        println!("{} = {}", var, line);
                    }
                }
            }
            Err(e) => {
                if !matches.get_flag("ignore") {
                    uucore::show!(e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let _matches: clap::ArgMatches = uu_app().try_get_matches_from(args)?;

    Err(uucore::error::USimpleError::new(
        1,
        "`sysctl` currently only supports Linux.",
    ))
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new("variables")
                .value_name("VARIABLE[=VALUE]")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .visible_short_aliases(['A', 'X'])
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Display all variables"),
        )
        .arg(
            Arg::new("names")
                .short('N')
                .long("names")
                .action(ArgAction::SetTrue)
                .help("Only print names"),
        )
        .arg(
            Arg::new("values")
                .short('n')
                .long("values")
                .action(ArgAction::SetTrue)
                .help("Only print values"),
        )
        .arg(
            Arg::new("ignore")
                .short('e')
                .long("ignore")
                .action(ArgAction::SetTrue)
                .help("Ignore errors"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Do not print when setting variables"),
        )
        .arg(
            Arg::new("noop_o")
                .short('o')
                .help("Does nothing, for BSD compatibility"),
        )
        .arg(
            Arg::new("noop_x")
                .short('x')
                .help("Does nothing, for BSD compatibility"),
        )
}
