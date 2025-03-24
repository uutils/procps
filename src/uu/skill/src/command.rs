use clap::ArgMatches;
use std::collections::HashSet;
use std::ffi::OsString;

pub const SIGNALS: &[&str] = &[
    "HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "BUS", "FPE", "KILL", "USR1", "SEGV", "USR2",
    "PIPE", "ALRM", "TERM", "STKFLT", "CHLD", "CONT", "STOP", "TSTP", "TTIN", "TTOU", "URG",
    "XCPU", "XFSZ", "VTALRM", "PROF", "WINCH", "POLL", "PWR", "SYS",
];
const DEFAULT_SIGNAL: &str = "TERM";
#[derive(Debug, Clone)]
pub struct Cli {
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

impl Cli {
    pub fn new(args: ArgMatches) -> Self {
        let signal = args.get_one::<String>("signal").unwrap().to_string();
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

pub fn parse_command(args: &mut impl uucore::Args) -> Vec<OsString> {
    let option_char_set: HashSet<char> =
        HashSet::from(['-', 'f', 'i', 'l', 'L', 'n', 'v', 'w', 'c', 'p', 't', 'u']);
    let args = args
        .map(|str| str.to_str().unwrap().into())
        .collect::<Vec<String>>();

    let exprs = |arg: &String| !arg.starts_with('-');
    let options =
        |arg: &String| arg.starts_with('-') && arg.chars().all(|c| option_char_set.contains(&c));

    let signals = args
        .iter()
        .filter(|arg0: &&String| !exprs(arg0))
        .filter(|arg1: &&String| !options(arg1))
        .collect::<Vec<&String>>();
    if signals.len() == 1 {
        let signal = signals[0];
        if SIGNALS.contains(&&signal.as_str()[1..]) {
            args.iter().map(|s| s.clone().into()).collect()
        } else {
            eprintln!("Invalid signal: {}", &signal.as_str()[1..]);
            std::process::exit(2);
        }
    } else if signals.is_empty() {
        // If no signal is provided, return the original args with default signal
        let mut new = args
            .iter()
            .map(|s| s.clone().into())
            .collect::<Vec<OsString>>();
        new.insert(1, ("-".to_string() + DEFAULT_SIGNAL).into());
        new
    } else {
        eprintln!("Too many signals");
        std::process::exit(2);
    }
}
