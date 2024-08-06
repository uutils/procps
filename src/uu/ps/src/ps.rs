// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod collector;
mod mapping;
mod parser;
mod picker;

use clap::crate_version;
use clap::{Arg, ArgAction, ArgMatches, Command};
use mapping::{collect_code_mapping, default_codes, default_mapping};
use parser::{parser, OptionalKeyValue};
use prettytable::{format::consts::FORMAT_CLEAN, Row, Table};
use std::{cell::RefCell, rc::Rc};
use uu_pgrep::process::walk_process;
use uucore::{
    error::{UError, UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("ps.md");
const USAGE: &str = help_usage!("ps.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let snapshot = walk_process()
        .map(|it| Rc::new(RefCell::new(it)))
        .collect::<Vec<_>>();
    let mut proc_infos = Vec::new();

    proc_infos.extend(collector::basic_collector(&snapshot));
    proc_infos.extend(collector::process_collector(&matches, &snapshot));
    proc_infos.extend(collector::session_collector(&matches, &snapshot));

    let arg_formats = collect_format(&matches);
    let Ok(arg_formats) = arg_formats else {
        return Err(arg_formats.err().unwrap());
    };

    // Collect codes with order
    let codes = if arg_formats.is_empty() {
        default_codes()
    } else {
        arg_formats.iter().map(|it| it.key().to_owned()).collect()
    };

    // Collect pickers ordered by codes
    let pickers = picker::collect_pickers(&codes);

    // Constructing table
    let mut rows = Vec::new();
    for proc in proc_infos {
        let picked = pickers
            .iter()
            .map(|picker| picker(Rc::unwrap_or_clone(proc.clone())));
        rows.push(Row::from_iter(picked));
    }

    // Apply header mapping
    let code_mapping = if arg_formats.is_empty() {
        let default_mapping = default_mapping();
        default_codes();
        codes
            .into_iter()
            .map(|code| (code.clone(), default_mapping[&code].to_string()))
            .collect::<Vec<_>>()
    } else {
        collect_code_mapping(&arg_formats)
    };

    let header = code_mapping
        .iter()
        .map(|(_, header)| header)
        .map(Into::into)
        .collect::<Vec<String>>();

    // Apply header
    let mut table = Table::from_iter([Row::from_iter(header)]);
    table.set_format(*FORMAT_CLEAN);
    table.extend(rows);

    // TODO: Sorting

    print!("{}", table);

    Ok(())
}

fn collect_format(
    matches: &ArgMatches,
) -> Result<Vec<OptionalKeyValue>, Box<dyn UError + 'static>> {
    let arg_format = matches.get_many::<OptionalKeyValue>("format");

    let collect = arg_format.unwrap_or_default().cloned().collect::<Vec<_>>();

    let default_mapping = default_mapping();

    // Validate key is exist
    for key in collect.iter().map(OptionalKeyValue::key) {
        if !default_mapping.contains_key(key) {
            return Err(USimpleError::new(
                1,
                format!("error: unknown user-defined format specifier \"{key}\""),
            ));
        }
    }

    Ok(collect)
}

#[allow(clippy::cognitive_complexity)]
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
        .arg(
            Arg::new("format")
                .short('o')
                .long("format")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .value_parser(parser)
                .help("user-defined format"),
        )
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
