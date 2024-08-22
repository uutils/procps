// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{thread::sleep, time::Duration};

use clap::{arg, crate_version, ArgAction, ArgGroup, ArgMatches, Command};
use picker::sysinfo;
use prettytable::{format::consts::FORMAT_CLEAN, Row, Table};
use sysinfo::{Pid, Users};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("top.md");
const USAGE: &str = help_usage!("top.md");

mod field;
mod picker;

#[allow(unused)]
#[derive(Debug)]
enum Filter {
    Pid(Vec<u32>),
    User(String),
    EUser(String),
}

#[derive(Debug)]
struct Settings {
    // batch:bool
    filter: Option<Filter>,
    width: Option<usize>,
}

impl Settings {
    fn new(matches: &ArgMatches) -> Self {
        let filter = if let Some(pidlist) = matches.get_many::<u32>("pid") {
            Some(Filter::Pid(pidlist.cloned().collect::<Vec<_>>()))
        } else if let Some(user) = matches.get_one::<String>("filter-any-user") {
            Some(Filter::User(user.clone()))
        } else {
            matches
                .get_one::<String>("filter-only-euser")
                .map(|euser| Filter::EUser(euser.clone()))
        };

        let width = matches.get_one::<usize>("width").cloned();

        Self { filter, width }
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Must refresh twice.
    picker::sysinfo().write().unwrap().refresh_all();
    sleep(Duration::from_millis(200));
    picker::sysinfo().write().unwrap().refresh_all();

    let settings = Settings::new(&matches);

    if let Some(Filter::User(user)) = &settings.filter {
        if Users::new_with_refreshed_list()
            .iter()
            .all(|it| it.name() != user)
        {
            return Err(USimpleError::new(1, "top: Invalid user"));
        }
    }

    let fields = selected_fields();
    let collected = collect(&settings, &fields);

    let table = {
        let mut table = Table::new();

        table.set_format(*FORMAT_CLEAN);

        table.add_row(Row::from_iter(fields));
        table.extend(collected.iter().map(Row::from_iter));

        table
    };

    println!("{}", header());
    println!("\n");

    let cutter = {
        #[inline]
        fn f(f: impl Fn(&str) -> String + 'static) -> Box<dyn Fn(&str) -> String> {
            Box::new(f)
        }

        if let Some(width) = settings.width {
            f(move |line: &str| apply_width(line, width))
        } else {
            f(|line: &str| line.to_string())
        }
    };

    table
        .to_string()
        .lines()
        .map(cutter)
        .for_each(|it| println!("{}", it));

    Ok(())
}

fn apply_width<T>(input: T, width: usize) -> String
where
    T: Into<String>,
{
    let input: String = input.into();

    if input.len() > width {
        input.chars().take(width).collect()
    } else {
        let mut result = String::from(&input);
        result.extend(std::iter::repeat(' ').take(width - input.len()));
        result
    }
}

// TODO: Implement information collecting.
fn header() -> String {
    "TODO".into()
}

// TODO: Implement fields selecting
fn selected_fields() -> Vec<String> {
    vec![
        "PID", "USER", "PR", "NI", "VIRT", "RES", "SHR", "S", "%CPU", "%MEM", "TIME+", "COMMAND",
    ]
    .into_iter()
    .map(Into::into)
    .collect()
}

fn collect(settings: &Settings, fields: &[String]) -> Vec<Vec<String>> {
    use picker::pickers;

    let pickers = pickers(fields);

    let pids = sysinfo()
        .read()
        .unwrap()
        .processes()
        .iter()
        .map(|(it, _)| it.as_u32())
        .collect::<Vec<_>>();

    let filter = construct_filter(settings);

    pids.into_iter()
        .filter(|pid| filter(*pid))
        .map(|it| {
            pickers
                .iter()
                .map(move |picker| picker(it))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Constructing filter from `Settings`
fn construct_filter(settings: &Settings) -> Box<dyn Fn(u32) -> bool> {
    let Some(ref filter) = settings.filter else {
        return Box::new(|_: u32| true);
    };

    fn helper(f: impl Fn(u32) -> bool + 'static) -> Box<dyn Fn(u32) -> bool> {
        Box::new(f)
    }

    match filter {
        Filter::Pid(pids) => {
            let pids = pids.clone();
            helper(move |pid: u32| pids.contains(&pid))
        }

        Filter::User(user) => {
            let user = user.to_owned();
            helper(move |pid| {
                let binding = sysinfo().read().unwrap();
                let Some(proc) = binding.process(Pid::from_u32(pid)) else {
                    return false;
                };

                let Some(uid) = proc.user_id() else {
                    return false;
                };

                Users::new_with_refreshed_list()
                    .get_user_by_id(uid)
                    .map(|it| it.name())
                    .unwrap_or("?")
                    == user
            })
        }
        // TODO: Implemented
        Filter::EUser(_) => helper(|_| true),
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            // arg!(-b  --"batch-mode"                         "run in non-interactive batch mode"),
            // arg!(-c  --"cmdline-toggle"                     "reverse last remembered 'c' state"),
            // arg!(-d  --delay                <SECS>          "iterative delay as SECS [.TENTHS]"),
            // arg!(-E  --"scale-summary-mem"  <SCALE>         "set mem as: k,m,g,t,p,e for SCALE"),
            // arg!(-e  --"scale-task-mem"     <SCALE>         "set mem with: k,m,g,t,p for SCALE"),
            // arg!(-H  --"threads-show"                       "show tasks plus all their threads"),
            // arg!(-i  --"idle-toggle"                        "reverse last remembered 'i' state"),
            // arg!(-n  --iterations           <NUMBER>        "exit on maximum iterations NUMBER"),
            arg!(-O  --"list-fields"                        "output all field names, then exit"),
            // arg!(-o  --"sort-override"      <FIELD>         "force sorting on this named FIELD"),
            arg!(-p  --pid                  <PIDLIST>       "monitor only the tasks in PIDLIST")
                .action(ArgAction::Append),
            // arg!(-S  --"accum-time-toggle"                  "reverse last remembered 'S' state"),
            // arg!(-s  --"secure-mode"                        "run with secure mode restrictions"),
            arg!(-U  --"filter-any-user"    <USER>          "show only processes owned by USER"),
            arg!(-u  --"filter-only-euser"  <EUSER>         "show only processes owned by USER"),
            arg!(-w  --width                <COLUMNS>       "change print width [,use COLUMNS]"),
            // arg!(-1  --single-cpu-toggle         "reverse last remembered '1' state"),
        ])
        .group(ArgGroup::new("filter").args(["pid", "filter-any-user", "filter-only-euser"]))
}
