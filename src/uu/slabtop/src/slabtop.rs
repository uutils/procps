// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep},
    time::Duration,
};

use crate::parse::SlabInfo;
use clap::{arg, crate_version, value_parser, ArgAction, ArgMatches, Command};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use parking_lot::RwLock;
use ratatui::widgets::Widget;
use tui::Tui;
use uucore::{error::UResult, format_usage, help_about, help_section, help_usage};

const ABOUT: &str = help_about!("slabtop.md");
const AFTER_HELP: &str = help_section!("after help", "slabtop.md");
const USAGE: &str = help_usage!("slabtop.md");

mod parse;
mod tui;

#[derive(Debug)]
struct Settings {
    pub(crate) delay: u64,
    pub(crate) once: bool,
    pub(crate) short_by: char,
}

impl Settings {
    fn new(arg: &ArgMatches) -> Settings {
        Settings {
            delay: *arg.get_one::<u64>("delay").unwrap_or(&3),
            once: arg.get_flag("once"),
            short_by: *arg.get_one::<char>("sort").unwrap_or(&'o'),
        }
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let settings = Settings::new(&matches);

    let slabinfo = Arc::new(RwLock::new(SlabInfo::new()?.sort(settings.short_by, false)));
    let should_update = Arc::new(AtomicBool::new(true));

    // Timer
    {
        let should_update = should_update.clone();
        thread::spawn(move || loop {
            sleep(Duration::from_secs(settings.delay));
            should_update.store(true, Ordering::Relaxed);
        });
    }
    // Update
    {
        let should_update = should_update.clone();
        let slabinfo = slabinfo.clone();
        thread::spawn(move || loop {
            if should_update.load(Ordering::Relaxed) {
                *slabinfo.write() = SlabInfo::new().unwrap().sort(settings.short_by, false);
                should_update.store(false, Ordering::Relaxed);
            }
            sleep(Duration::from_millis(20));
        });
    }

    let mut terminal = ratatui::init();
    loop {
        if let Ok(true) = event::poll(Duration::from_millis(10)) {
            // If event available, break this loop
            if let Ok(e) = event::read() {
                match e {
                    event::Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    })
                    | event::Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => {
                        uucore::error::set_exit_code(0);
                        break;
                    }
                    event::Event::Key(KeyEvent {
                        code: KeyCode::Char(' '),
                        ..
                    }) => should_update.store(true, Ordering::Relaxed),
                    _ => {}
                }
            }
        }

        terminal.draw(|frame| {
            Tui::new(&slabinfo.read()).render(frame.area(), frame.buffer_mut());
        })?;

        if settings.once {
            break;
        } else {
            sleep(Duration::from_millis(10));
        }
    }

    if !settings.once {
        ratatui::restore();
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            arg!(-d --delay <secs>  "delay updates")
                .value_parser(value_parser!(u64))
                .default_value("3"),
            arg!(-o --once          "only display once, then exit").action(ArgAction::SetTrue),
            arg!(-s --sort  <char>  "specify sort criteria by character (see below)")
                .value_parser(value_parser!(char)),
        ])
        .after_help(AFTER_HELP)
}
