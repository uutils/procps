// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::thread::{self, sleep};
use std::time::Duration;

use clap::{arg, crate_version, value_parser, ArgAction, ArgMatches, Command};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use tui::{LegacyTui, ModernTui};
use uucore::error::UResult;

mod tui;

#[derive(Debug, Default, Clone)]
struct SystemLoadAvg {
    pub(crate) last_1: f32,
    pub(crate) last_5: f32,
    pub(crate) last_10: f32,
}

impl SystemLoadAvg {
    #[cfg(target_os = "linux")]
    fn new() -> UResult<SystemLoadAvg> {
        use std::fs;
        use uucore::error::USimpleError;

        let result = fs::read_to_string("/proc/loadavg")?;
        let split = result.split(" ").collect::<Vec<_>>();

        // Helper function to keep code clean
        fn f(s: &str) -> UResult<f32> {
            s.parse::<f32>()
                .map_err(|e| USimpleError::new(1, e.to_string()))
        }

        Ok(SystemLoadAvg {
            last_1: f(split[0])?,
            last_5: f(split[1])?,
            last_10: f(split[2])?,
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn new() -> UResult<SystemLoadAvg> {
        Ok(SystemLoadAvg::default())
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Settings {
    delay: u64,
    scale: usize, // Not used

    is_modern: bool, // For modern display
}

impl Settings {
    fn new(matches: &ArgMatches) -> Settings {
        Settings {
            delay: matches.get_one("delay").cloned().unwrap(),
            scale: matches.get_one("scale").cloned().unwrap(),
            is_modern: matches.get_flag("modern"),
        }
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let settings = Settings::new(&matches);

    let mut terminal = ratatui::init();

    let data = {
        // Why 10240?
        //
        // Emm, maybe there will be some terminal can display more than 10000 char?
        let data = Arc::new(RwLock::new(VecDeque::with_capacity(10240)));
        data.write()
            .unwrap()
            .push_back(SystemLoadAvg::new().unwrap());
        data
    };
    let cloned_data = data.clone();
    thread::spawn(move || loop {
        sleep(Duration::from_secs(settings.delay));

        let mut data = cloned_data.write().unwrap();
        if data.iter().len() >= 10240 {
            // Keep this VecDeque smaller than 10240
            data.pop_front();
        }
        data.push_back(SystemLoadAvg::new().unwrap());
    });

    loop {
        // Now only accept `Ctrl+C` for compatibility with the original implementation
        //
        // Use `event::poll` for non-blocking event reading
        if let Ok(true) = event::poll(Duration::from_millis(10)) {
            // If event available, break this loop
            if let Ok(event::Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })) = event::read()
            {
                // compatibility with the original implementation
                uucore::error::set_exit_code(130);
                break;
            }
        }

        terminal.draw(|frame| {
            let data = &data.read().unwrap();
            let data = data.iter().cloned().collect::<Vec<_>>();
            frame.render_widget(
                if settings.is_modern {
                    ModernTui::new(&data)
                } else {
                    LegacyTui::new(&data)
                },
                frame.area(),
            );
        })?;

        std::thread::sleep(Duration::from_millis(10));
    }

    ratatui::restore();
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("tload prints a graph of the current system load average to the specified tty (or the tty of the tload process if none is specified)")
        .override_usage("tload [options] [tty]")
        .infer_long_args(true)
        .args([
            arg!(-d --delay     <secs>  "update delay in seconds")
                .value_parser(value_parser!(u64))
                .default_value("5")
                .hide_default_value(true),
            arg!(-m --modern            "modern look").action(ArgAction::SetTrue),
            // TODO: Implement this arg
            arg!(-s --scale <num>       "vertical scale")
                .value_parser(value_parser!(usize))
                .default_value("5")
                .hide_default_value(true),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    // It's just a test to make sure if can parsing correctly.
    #[test]
    fn test_system_load_avg() {
        let _ = SystemLoadAvg::new().expect("SystemLoadAvg::new");
    }
}
