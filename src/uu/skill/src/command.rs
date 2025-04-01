// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use std::collections::HashSet;
use uucore::signals::{ALL_SIGNALS, DEFAULT_SIGNAL};

#[derive(Debug, Clone)]
pub struct Settings {
    // Arguments
    pub signal: String,
    pub expression: Expr,
    // Flags
    pub fast: bool,
    pub interactive: bool,
    pub list: bool,
    pub table: bool,
    pub no_action: bool,
    pub verbose: bool,
    pub warnings: bool,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Terminal(Vec<String>),
    User(Vec<String>),
    Pid(Vec<i32>),
    Command(Vec<String>),
    Raw(Vec<String>),
}

impl Settings {
    pub fn new(args: ArgMatches) -> Self {
        let mut signal = args.get_one::<String>("signal").unwrap().to_string();
        if signal.starts_with("-") {
            signal.remove(0);
        }
        let literal: Vec<String> = args
            .get_many("expression")
            .unwrap_or_default()
            .cloned()
            .collect();
        let expr = if args.get_flag("command") {
            Expr::Command(literal)
        } else if args.get_flag("user") {
            Expr::User(literal)
        } else if args.get_flag("pid") {
            Expr::Pid(literal.iter().map(|s| s.parse::<i32>().unwrap()).collect())
        } else if args.get_flag("tty") {
            Expr::Terminal(literal)
        } else {
            Expr::Raw(literal)
        };

        Self {
            signal,
            expression: expr,
            fast: args.get_flag("fast"),
            interactive: args.get_flag("interactive"),
            list: args.get_flag("list"),
            table: args.get_flag("table"),
            no_action: args.get_flag("no-action"),
            verbose: args.get_flag("verbose"),
            warnings: args.get_flag("warnings"),
        }
    }
}

// Pre-parses the command line arguments and returns a vector of OsString

// Mainly used to parse the signal to make sure it is valid
// and insert the default signal if it's not present
pub fn parse_command(args: &mut impl uucore::Args) -> Vec<String> {
    let option_char_set: HashSet<char> = HashSet::from([
        '-', 'f', 'i', 'l', 'L', 'n', 'v', 'w', 'c', 'p', 't', 'u', 'h', 'V',
    ]);
    let option_set: HashSet<&str> = HashSet::from([
        "--table",
        "--list",
        "--no-action",
        "--verbose",
        "--warnings",
        "--interactive",
        "--fast",
        "--command",
        "--user",
        "--pid",
        "--tty",
        "--help",
        "--version",
    ]);
    let args = args
        .map(|str| str.to_str().unwrap().into())
        .collect::<Vec<String>>();

    let exprs = |arg: &String| !arg.starts_with('-');
    let options = |arg: &String| {
        (arg.starts_with('-') && arg.chars().all(|c| option_char_set.contains(&c)))
            || (arg.starts_with("--") && option_set.contains(&arg.as_str()))
    };

    let signals = args
        .iter()
        .filter(|arg: &&String| !exprs(arg))
        .filter(|arg: &&String| !options(arg))
        .collect::<Vec<&String>>();
    if signals.len() == 1 {
        let signal = &signals[0].as_str()[1..]; // Remove the leading '-'
        if ALL_SIGNALS.contains(&signal) {
            args.to_vec()
        } else {
            eprintln!("Invalid signal: {}", &signal);
            std::process::exit(2);
        }
    } else if signals.is_empty() {
        // If no signal is provided, return the original args with default signal
        let mut new = args.to_vec();
        new.insert(1, ALL_SIGNALS[DEFAULT_SIGNAL].to_string());
        new
    } else {
        eprintln!("Too many signals");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod test {
    use super::parse_command;
    use std::ffi::OsString;

    #[test]
    fn test_parse_command_normal() {
        let args: Vec<OsString> = vec!["skill", "-TERM", "-v", "1234"]
            .into_iter()
            .map(|s| OsString::from(s))
            .collect();
        let parsed = parse_command(&mut args.iter().map(|s| s.clone()));
        assert_eq!(parsed, vec!["skill", "-TERM", "-v", "1234"]);
    }

    #[test]
    fn test_parse_command_default_signal() {
        let args: Vec<OsString> = vec!["skill", "-v", "-l", "1234"]
            .into_iter()
            .map(|s| OsString::from(s))
            .collect();
        let parsed = parse_command(&mut args.iter().cloned());
        assert_eq!(parsed, vec!["skill", "TERM", "-v", "-l", "1234"]);
    }

    #[test]
    fn test_parse_command_unordered() {
        let args: Vec<OsString> = vec!["skill", "-v", "-l", "-KILL", "1234"]
            .into_iter()
            .map(|s| OsString::from(s))
            .collect();
        let parsed = parse_command(&mut args.iter().map(|s| s.clone()));
        assert_eq!(parsed, vec!["skill", "-v", "-l", "-KILL", "1234"]);
    }
}
