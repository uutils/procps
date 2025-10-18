// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::header::Header;
use crate::picker::get_command;
use crate::platform::get_numa_nodes;
use crate::tui::stat::{CpuValueMode, TuiStat};
use crate::Filter::{EUser, User};
use crate::{selected_fields, try_into_uid, InfoBar, ProcList, Settings};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum InputMode {
    Command,
    Input(InputEvent),
}
#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum InputEvent {
    MaxListDisplay,
    NumaNode,
    FilterUser,
    FilterEUser,
    WidthIncrement,
    Delay,
}

macro_rules! char {
    ($e:expr) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($e),
            ..
        })
    };
}

pub fn handle_input(
    e: Event,
    settings: &Settings,
    tui_stat: &RwLock<TuiStat>,
    data: &RwLock<(Header, ProcList, Option<InfoBar>)>,
    should_update: &AtomicBool,
) -> bool {
    let input_mode = { tui_stat.read().unwrap().input_mode };
    match input_mode {
        InputMode::Command => match e {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
            | char!('q') => {
                uucore::error::set_exit_code(0);
                return true;
            }

            char!('b') => {
                let mut stat = tui_stat.write().unwrap();
                stat.highlight_bold = !stat.highlight_bold;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('C') => {
                let mut stat = tui_stat.write().unwrap();
                stat.show_coordinates = !stat.show_coordinates;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('c') => {
                {
                    // drop the lock as soon as possible
                    let mut stat = tui_stat.write().unwrap();
                    stat.full_command_line = !stat.full_command_line;
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            char!('d') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = format!("Change delay from {:.1} to ", stat.delay.as_secs_f32());
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::Delay);

                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                {
                    let mut stat = tui_stat.write().unwrap();
                    stat.time_scale = stat.time_scale.next();
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            #[cfg(target_os = "linux")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                let mut data = data.write().unwrap();
                if data.2.is_some() {
                    data.2 = None;
                } else {
                    let tui_stat = tui_stat.read().unwrap();
                    let mut nth = tui_stat.list_offset;
                    if data.1.collected.is_empty() {
                        return false;
                    }
                    if data.1.collected.len() <= nth {
                        nth = data.1.collected.len() - 1;
                    }
                    let pid = data.1.collected[nth].0;
                    let title = format!(
                        "control groups for pid {}, {}",
                        pid,
                        get_command(pid, false)
                    );
                    let content = crate::picker::get_cgroup(pid);
                    data.2 = Some(InfoBar { title, content });
                }
                should_update.store(true, Ordering::Relaxed);
            }
            char!('I') => {
                {
                    let mut stat = tui_stat.write().unwrap();
                    stat.irix_mode = !stat.irix_mode;
                    stat.input_message = Some(format!(
                        " Irix mode {} ",
                        if stat.irix_mode { "On" } else { "Off" }
                    ));
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                let mut data = data.write().unwrap();
                if data.2.is_some() {
                    data.2 = None;
                } else {
                    let tui_stat = tui_stat.read().unwrap();
                    let mut nth = tui_stat.list_offset;
                    if data.1.collected.is_empty() {
                        return false;
                    }
                    if data.1.collected.len() <= nth {
                        nth = data.1.collected.len() - 1;
                    }
                    let pid = data.1.collected[nth].0;
                    let title =
                        format!("command line for pid {}, {}", pid, get_command(pid, false));
                    let content = get_command(pid, true);
                    data.2 = Some(InfoBar { title, content });
                }
                should_update.store(true, Ordering::Relaxed);
            }
            char!('l') => {
                let mut stat = tui_stat.write().unwrap();
                stat.show_load_avg = !stat.show_load_avg;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('m') => {
                let mut stat = tui_stat.write().unwrap();
                stat.memory_graph_mode = stat.memory_graph_mode.next();
                should_update.store(true, Ordering::Relaxed);
            }
            char!('n') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = format!(
                    "Maximum tasks = {}, change to (0 is unlimited)",
                    stat.max_list_display
                );
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::MaxListDisplay);

                should_update.store(true, Ordering::Relaxed);
            }
            char!('R') => {
                {
                    let mut stat = tui_stat.write().unwrap();
                    stat.sort_by_pid = !stat.sort_by_pid;
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            char!('t') => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_graph_mode = stat.cpu_graph_mode.next();
                should_update.store(true, Ordering::Relaxed);
            }
            #[cfg(target_os = "linux")]
            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                let mut data = data.write().unwrap();
                if data.2.is_some() {
                    data.2 = None;
                } else {
                    let tui_stat = tui_stat.read().unwrap();
                    let mut nth = tui_stat.list_offset;
                    if data.1.collected.is_empty() {
                        return false;
                    }
                    if data.1.collected.len() <= nth {
                        nth = data.1.collected.len() - 1;
                    }
                    let pid = data.1.collected[nth].0;
                    let title = format!(
                        "supplementary groups for pid {}, {}",
                        pid,
                        get_command(pid, false)
                    );
                    let content = crate::picker::get_supplementary_groups(pid);
                    data.2 = Some(InfoBar { title, content });
                }
                should_update.store(true, Ordering::Relaxed);
            }
            char!('U') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = "Which user (blank for all) ".into();
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::FilterUser);

                should_update.store(true, Ordering::Relaxed);
            }
            char!('u') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = "Which user (blank for all) ".into();
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::FilterEUser);

                should_update.store(true, Ordering::Relaxed);
            }
            char!('X') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = "width incr is 0, change to (0 default, -1 auto) ".into();
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::WidthIncrement);

                should_update.store(true, Ordering::Relaxed);
            }
            char!('x') => {
                let mut stat = tui_stat.write().unwrap();
                stat.highlight_sorted = !stat.highlight_sorted;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('z') => {
                let mut stat = tui_stat.write().unwrap();
                stat.colorful = !stat.colorful;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('0') => {
                {
                    // drop the lock as soon as possible
                    let mut stat = tui_stat.write().unwrap();
                    stat.show_zeros = !stat.show_zeros;
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            char!('1') => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_value_mode = stat.cpu_value_mode.next();

                should_update.store(true, Ordering::Relaxed);
                data.write().unwrap().0.update_cpu(&stat);
            }
            char!('2') => {
                let mut stat = tui_stat.write().unwrap();
                if stat.cpu_value_mode == CpuValueMode::Numa {
                    stat.cpu_value_mode = stat.cpu_value_mode.next();
                } else {
                    stat.cpu_value_mode = CpuValueMode::Numa;
                    stat.cpu_column = 1;
                }

                data.write().unwrap().0.update_cpu(&stat);
                should_update.store(true, Ordering::Relaxed);
            }
            char!('3') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = "expand which numa node ".into();
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::NumaNode);

                should_update.store(true, Ordering::Relaxed);
                data.write().unwrap().0.update_cpu(&stat);
            }
            char!('4') => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_column = stat.cpu_column % 8 + 1;
                should_update.store(true, Ordering::Relaxed);
            }
            char!('#') => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = format!(
                    "Maximum tasks = {}, change to (0 is unlimited)",
                    stat.max_list_display
                );
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::MaxListDisplay);

                should_update.store(true, Ordering::Relaxed);
            }
            char!('<') => {
                {
                    let mut stat = tui_stat.write().unwrap();
                    let fields = selected_fields();
                    if let Some(pos) = fields.iter().position(|f| f == &stat.sorter) {
                        let new_pos = if pos == 0 { pos } else { pos - 1 };
                        stat.sorter = fields[new_pos].clone();
                    } else {
                        stat.sorter = fields[0].clone();
                    }
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            char!('>') => {
                {
                    let mut stat = tui_stat.write().unwrap();
                    let fields = selected_fields();
                    if let Some(pos) = fields.iter().position(|f| f == &stat.sorter) {
                        let new_pos = if pos + 1 >= fields.len() {
                            pos
                        } else {
                            pos + 1
                        };
                        stat.sorter = fields[new_pos].clone();
                    } else {
                        stat.sorter = fields[0].clone();
                    }
                }

                data.write().unwrap().1 = ProcList::new(settings, &tui_stat.read().unwrap());
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                if stat.list_offset > 0 {
                    stat.list_offset -= 1;
                    should_update.store(true, Ordering::Relaxed);
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.list_offset += 1;
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                if stat.horizontal_offset > 0 {
                    stat.horizontal_offset -= 1;
                    should_update.store(true, Ordering::Relaxed);
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.horizontal_offset += 1;
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Resize(_, _) => should_update.store(true, Ordering::Relaxed),
            _ => {}
        },
        InputMode::Input(input_event) => {
            if let Event::Key(key) = e {
                match key.code {
                    KeyCode::Enter => {
                        handle_input_value(input_event, settings, tui_stat, data, should_update);
                    }
                    KeyCode::Esc => {
                        let mut stat = tui_stat.write().unwrap();
                        stat.reset_input();
                        should_update.store(true, Ordering::Relaxed);
                    }
                    KeyCode::Backspace => {
                        let mut app = tui_stat.write().unwrap();
                        app.input_value.pop();
                        should_update.store(true, Ordering::Relaxed);
                    }
                    KeyCode::Char(c) => {
                        let mut app = tui_stat.write().unwrap();
                        app.input_value.push(c);
                        should_update.store(true, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        }
    }
    false
}

fn handle_input_value(
    input_event: InputEvent,
    settings: &Settings,
    tui_stat: &RwLock<TuiStat>,
    data: &RwLock<(Header, ProcList, Option<InfoBar>)>,
    should_update: &AtomicBool,
) {
    match input_event {
        InputEvent::MaxListDisplay => {
            let input_value = { tui_stat.read().unwrap().input_value.parse::<usize>() };
            if input_value.is_err() {
                let mut stat = tui_stat.write().unwrap();
                stat.reset_input();
                stat.input_message = Some(" invalid number ".into());
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let input_value = input_value.unwrap();
            let mut stat = tui_stat.write().unwrap();
            stat.max_list_display = input_value;
            stat.reset_input();
            should_update.store(true, Ordering::Relaxed);
        }
        InputEvent::NumaNode => {
            let input_value = { tui_stat.read().unwrap().input_value.parse::<usize>() };
            let numa_nodes = get_numa_nodes();
            if input_value.is_err()
                || input_value
                    .as_ref()
                    .is_ok_and(|v| !numa_nodes.contains_key(v))
            {
                let mut stat = tui_stat.write().unwrap();
                stat.reset_input();
                stat.input_message = Some(" invalid numa node ".into());
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let input_value = input_value.unwrap();
            let mut stat = tui_stat.write().unwrap();
            stat.cpu_value_mode = CpuValueMode::NumaNode(input_value);
            stat.cpu_column = 1;
            stat.reset_input();
            data.write().unwrap().0.update_cpu(&stat);
            should_update.store(true, Ordering::Relaxed);
        }
        InputEvent::FilterUser | InputEvent::FilterEUser => {
            let input_value = { tui_stat.read().unwrap().input_value.clone() };
            if input_value.is_empty() {
                let mut stat = tui_stat.write().unwrap();
                stat.filter = None;
                data.write().unwrap().1 = ProcList::new(settings, &stat);
                stat.reset_input();
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let user = match try_into_uid(&input_value) {
                Ok(user) => user,
                Err(_) => {
                    let mut stat = tui_stat.write().unwrap();
                    stat.reset_input();
                    stat.input_message = Some(" invalid user ".into());
                    should_update.store(true, Ordering::Relaxed);
                    return;
                }
            };

            let mut stat = tui_stat.write().unwrap();
            match input_event {
                InputEvent::FilterUser => {
                    stat.filter = Some(User(user));
                }
                InputEvent::FilterEUser => {
                    stat.filter = Some(EUser(user));
                }
                _ => {}
            }
            data.write().unwrap().1 = ProcList::new(settings, &stat);
            stat.reset_input();
            should_update.store(true, Ordering::Relaxed);
        }
        InputEvent::WidthIncrement => {
            let input_value = { tui_stat.read().unwrap().input_value.parse::<isize>() };

            if input_value.is_err() || input_value.as_ref().is_ok_and(|v| *v < -1) {
                let is_empty = { tui_stat.read().unwrap().input_value.trim().is_empty() };
                let mut stat = tui_stat.write().unwrap();
                stat.reset_input();
                if !is_empty {
                    stat.input_message = Some(" Unacceptable integer ".into());
                }
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let input_value = input_value.unwrap();
            let mut stat = tui_stat.write().unwrap();
            stat.width_increment = if input_value == -1 {
                None
            } else {
                Some(input_value as usize)
            };
            stat.reset_input();
            should_update.store(true, Ordering::Relaxed);
        }
        InputEvent::Delay => {
            let input_value = { tui_stat.read().unwrap().input_value.parse::<f32>() };
            if input_value.is_err() || input_value.as_ref().is_ok_and(|v| *v < 0.0) {
                let is_empty = { tui_stat.read().unwrap().input_value.trim().is_empty() };
                let mut stat = tui_stat.write().unwrap();
                stat.reset_input();
                if !is_empty {
                    stat.input_message = Some(" Unacceptable floating point ".into());
                }
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let input_value = input_value.unwrap();
            let mut stat = tui_stat.write().unwrap();
            stat.delay = std::time::Duration::from_secs_f32(input_value);
            stat.reset_input();
            should_update.store(true, Ordering::Relaxed);
        }
    }
}
