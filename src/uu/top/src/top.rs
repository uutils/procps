// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::header::Header;
use crate::tui::stat::TuiStat;
use crate::tui::{handle_input, Tui};
use clap::{arg, crate_version, value_parser, ArgAction, ArgGroup, ArgMatches, Command};
use picker::pickers;
use picker::sysinfo;
use ratatui::crossterm::event;
use ratatui::prelude::Widget;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::{thread, thread::sleep, time::Duration};
use sysinfo::{Pid, Users};
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("top.md");
const USAGE: &str = help_usage!("top.md");

mod field;
mod header;
mod picker;
mod platform;
mod tui;

#[allow(unused)]
#[derive(Debug)]
pub enum Filter {
    Pid(Vec<u32>),
    User(String),
    EUser(String),
}

#[derive(Debug)]
pub(crate) struct Settings {
    // batch:bool
    filter: Option<Filter>,
    scale_summary_mem: Option<String>,
}

impl Settings {
    fn new(matches: &ArgMatches) -> Self {
        Self {
            filter: None,
            scale_summary_mem: matches.get_one::<String>("scale-summary-mem").cloned(),
        }
    }
}

pub(crate) struct ProcList {
    pub fields: Vec<String>,
    pub collected: Vec<Vec<String>>,
}

impl ProcList {
    pub fn new(settings: &Settings, tui_stat: &TuiStat) -> Self {
        let fields = selected_fields();
        let collected = collect(settings, &fields, tui_stat);

        Self { fields, collected }
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Must refresh twice.
    // https://docs.rs/sysinfo/0.31.2/sysinfo/struct.System.html#method.refresh_cpu_usage
    picker::sysinfo().write().unwrap().refresh_all();
    sleep(Duration::from_millis(200));
    picker::sysinfo().write().unwrap().refresh_all();

    let settings = Settings::new(&matches);

    let settings = {
        let filter = matches
            .get_many::<u32>("pid")
            .map(|pidlist| Filter::Pid(pidlist.cloned().collect()))
            .or_else(|| {
                matches
                    .get_one::<String>("filter-any-user")
                    .map(|user| Filter::User(user.clone()))
            })
            .or_else(|| {
                matches
                    .get_one::<String>("filter-only-euser")
                    .map(|euser| Filter::EUser(euser.clone()))
            });

        let filter = match filter {
            Some(Filter::User(data)) => Some(Filter::User(try_into_uid(data)?)),
            // TODO: Make sure this working
            Some(Filter::EUser(data)) => Some(Filter::EUser(try_into_uid(data)?)),
            _ => filter,
        };

        Settings { filter, ..settings }
    };

    let settings = Arc::new(settings);
    let tui_stat = Arc::new(RwLock::new(TuiStat::new()));
    let should_update = Arc::new(AtomicBool::new(true));
    let data = Arc::new(RwLock::new((
        Header::new(&tui_stat.read().unwrap()),
        ProcList::new(&settings, &tui_stat.read().unwrap()),
    )));

    // update
    {
        let should_update = should_update.clone();
        let tui_stat = tui_stat.clone();
        let data = data.clone();
        let settings = settings.clone();
        thread::spawn(move || loop {
            let delay = { tui_stat.read().unwrap().delay };
            sleep(delay);
            {
                let header = Header::new(&tui_stat.read().unwrap());
                let proc_list = ProcList::new(&settings, &tui_stat.read().unwrap());
                tui_stat.write().unwrap().input_error = None;
                *data.write().unwrap() = (header, proc_list);
                should_update.store(true, Ordering::Relaxed);
            }
        });
    }

    let mut terminal = ratatui::init();
    terminal.draw(|frame| {
        Tui::new(
            &settings,
            &data.read().unwrap(),
            &mut tui_stat.write().unwrap(),
        )
        .render(frame.area(), frame.buffer_mut());
    })?;
    loop {
        if let Ok(true) = event::poll(Duration::from_millis(20)) {
            if let Ok(e) = event::read() {
                if handle_input(e, &settings, &tui_stat, &data, &should_update) {
                    break;
                }
            }
        }

        if should_update.load(Ordering::Relaxed) {
            terminal.draw(|frame| {
                Tui::new(
                    &settings,
                    &data.read().unwrap(),
                    &mut tui_stat.write().unwrap(),
                )
                .render(frame.area(), frame.buffer_mut());
            })?;
        }
        should_update.store(false, Ordering::Relaxed);
    }

    ratatui::restore();

    Ok(())
}

fn try_into_uid<T>(input: T) -> UResult<String>
where
    T: Into<String>,
{
    let into: String = input.into();

    if into.parse::<u32>().is_ok() {
        return Ok(into);
    }

    let user_name = into;
    let users = Users::new_with_refreshed_list();

    users
        .iter()
        .find(|it| it.name() == user_name)
        .map(|it| it.id().to_string())
        .ok_or(USimpleError::new(1, "Invalid user"))
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

fn collect(settings: &Settings, fields: &[String], tui_stat: &TuiStat) -> Vec<Vec<String>> {
    let pickers = pickers(fields);

    let pids = sysinfo()
        .read()
        .unwrap()
        .processes()
        .keys()
        .map(|it| it.as_u32())
        .collect::<Vec<_>>();

    let filter = construct_filter(settings);

    pids.into_iter()
        .filter(|pid| filter(*pid))
        .map(|it| {
            pickers
                .iter()
                .map(move |picker| picker(it, (settings, tui_stat)))
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

                uid.to_string() == user
            })
        }

        // Doesn't work on Windows.
        // https://docs.rs/sysinfo/0.31.3/sysinfo/struct.Process.html#method.effective_user_id
        Filter::EUser(euser) => {
            let euser = euser.to_owned();

            helper(move |pid| {
                let binding = sysinfo().read().unwrap();
                let Some(proc) = binding.process(Pid::from_u32(pid)) else {
                    return false;
                };

                let Some(euid) = proc.effective_user_id() else {
                    return false;
                };

                euid.to_string() == euser
            })
        }
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
            arg!(-E  --"scale-summary-mem"  <SCALE>         "set mem as: k,m,g,t,p,e for SCALE"),
            // arg!(-e  --"scale-task-mem"     <SCALE>         "set mem with: k,m,g,t,p for SCALE"),
            // arg!(-H  --"threads-show"                       "show tasks plus all their threads"),
            // arg!(-i  --"idle-toggle"                        "reverse last remembered 'i' state"),
            // arg!(-n  --iterations           <NUMBER>        "exit on maximum iterations NUMBER"),
            arg!(-O  --"list-fields"                        "output all field names, then exit"),
            // arg!(-o  --"sort-override"      <FIELD>         "force sorting on this named FIELD"),
            arg!(-p  --pid                  <PIDLIST>       "monitor only the tasks in PIDLIST")
                .action(ArgAction::Append)
                .value_parser(value_parser!(u32))
                .value_delimiter(','),
            // arg!(-S  --"accum-time-toggle"                  "reverse last remembered 'S' state"),
            // arg!(-s  --"secure-mode"                        "run with secure mode restrictions"),
            arg!(-U  --"filter-any-user"    <USER>          "show only processes owned by USER"),
            arg!(-u  --"filter-only-euser"  <EUSER>         "show only processes owned by USER"),
            // arg!(-w  --width                <COLUMNS>       "change print width [,use COLUMNS]"),
            // arg!(-1  --single-cpu-toggle         "reverse last remembered '1' state"),
        ])
        .group(ArgGroup::new("filter").args(["pid", "filter-any-user", "filter-only-euser"]))
}
