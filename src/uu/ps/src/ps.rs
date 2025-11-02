// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod mapping;
mod parser;
mod picker;
mod process_selection;
mod sorting;

use clap::crate_version;
use clap::{Arg, ArgAction, ArgMatches, Command};
use mapping::{
    collect_code_mapping, default_codes, default_mapping, default_with_psr_codes,
    extra_full_format_codes, full_format_codes, job_format_codes, long_format_codes,
    long_y_format_codes, register_format_codes, signal_format_codes, user_format_codes,
    vm_format_codes,
};
use parser::{parser, OptionalKeyValue};
use prettytable::{format::consts::FORMAT_CLEAN, Row, Table};
use process_selection::ProcessSelectionSettings;
use std::cell::RefCell;
use uucore::{
    error::{UError, UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("ps.md");
const USAGE: &str = help_usage!("ps.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let selection_settings = ProcessSelectionSettings::from_matches(&matches);
    let mut proc_infos = selection_settings.select_processes()?;
    if proc_infos.is_empty() {
        uucore::error::set_exit_code(1);
    }

    sorting::sort(&mut proc_infos, &matches);

    let arg_formats = collect_format(&matches);
    let Ok(arg_formats) = arg_formats else {
        return Err(arg_formats.err().unwrap());
    };

    // Collect codes with order
    let codes = if matches.get_flag("f") {
        full_format_codes()
    } else if matches.get_flag("F") {
        extra_full_format_codes()
    } else if matches.get_flag("j") {
        job_format_codes()
    } else if matches.get_flag("l") && matches.get_flag("y") {
        long_y_format_codes()
    } else if matches.get_flag("l") {
        long_format_codes()
    } else if matches.get_flag("P") {
        default_with_psr_codes()
    } else if matches.get_flag("s") {
        signal_format_codes()
    } else if matches.get_flag("u") {
        user_format_codes()
    } else if matches.get_flag("v") {
        vm_format_codes()
    } else if matches.get_flag("X") {
        register_format_codes()
    } else if arg_formats.is_empty() {
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
            .map(|picker| picker(RefCell::new(proc.clone())));
        rows.push(Row::from_iter(picked));
    }

    // Apply header mapping
    let code_mapping = if arg_formats.is_empty() {
        let default_mapping = default_mapping();
        default_codes();
        codes
            .into_iter()
            .map(|code| (code.clone(), default_mapping[&code].clone()))
            .collect::<Vec<_>>()
    } else {
        collect_code_mapping(&arg_formats)
    };

    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);
    if !matches.get_flag("no-headers") {
        let header = code_mapping
            .iter()
            .map(|(_, header)| header)
            .map(Into::into)
            .collect::<Vec<String>>();
        table.add_row(Row::from_iter(header));
    }
    table.extend(rows);

    print!("{table}");

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
                .visible_short_alias('e')
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
            Arg::new("f")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("full format listing"),
        )
        .arg(
            Arg::new("F")
                .short('F')
                .action(ArgAction::SetTrue)
                .help("extra full format listing"),
        )
        .arg(
            Arg::new("j")
                .short('j')
                .action(ArgAction::SetTrue)
                .help("job format"),
        )
        .arg(
            Arg::new("l")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("long format"),
        )
        .arg(
            Arg::new("P")
                .short('P')
                .action(ArgAction::SetTrue)
                .help("add psr column"),
        )
        .arg(
            Arg::new("s")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("signal format"),
        )
        // TODO: this can also be used with argument to filter by uid
        .arg(
            Arg::new("u")
                .short('u')
                .action(ArgAction::SetTrue)
                .help("user format"),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("virtual memory format"),
        )
        .arg(
            Arg::new("y")
                .short('y')
                .action(ArgAction::SetTrue)
                .help("do not show flags, show rss vs. addr (used with -l)"),
        )
        .arg(
            Arg::new("X")
                .short('X')
                .action(ArgAction::SetTrue)
                .help("register format"),
        )
        .arg(
            Arg::new("format")
                .short('o')
                .long("format")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .value_parser(parser)
                .help("user-defined format"),
        )
        .arg(
            Arg::new("no-headers")
                .long("no-headers")
                .visible_alias("no-heading")
                .action(ArgAction::SetTrue)
                .help("do not print header at all"),
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
