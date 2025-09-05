// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::header::Header;
use crate::platform::get_numa_nodes;
use crate::tui::stat::{CpuValueMode, TuiStat};
use crate::ProcList;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use uucore::error::UResult;

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum InputMode {
    Command,
    Input(InputEvent),
}
#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum InputEvent {
    NumaNode,
}

pub fn handle_input(
    e: Event,
    tui_stat: &RwLock<TuiStat>,
    data: &RwLock<(Header, ProcList)>,
    should_update: &AtomicBool,
) -> UResult<bool> {
    let input_mode = { tui_stat.read().unwrap().input_mode };
    match input_mode {
        InputMode::Command => match e {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                uucore::error::set_exit_code(0);
                return Ok(true);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.show_load_avg = !stat.show_load_avg;
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_graph_mode = stat.cpu_graph_mode.next();
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_value_mode = stat.cpu_value_mode.next();

                should_update.store(true, Ordering::Relaxed);
                data.write().unwrap().0.update_cpu(&stat);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                if stat.cpu_value_mode != CpuValueMode::Numa {
                    stat.cpu_value_mode = CpuValueMode::Numa;
                    stat.cpu_column = 1;
                } else {
                    stat.cpu_value_mode = stat.cpu_value_mode.next();
                }

                data.write().unwrap().0.update_cpu(&stat);
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.input_label = "expand which numa node ".into();
                stat.input_value.clear();
                stat.input_mode = InputMode::Input(InputEvent::NumaNode);

                should_update.store(true, Ordering::Relaxed);
                data.write().unwrap().0.update_cpu(&stat);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('4'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.cpu_column = stat.cpu_column % 8 + 1;
                should_update.store(true, Ordering::Relaxed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('m'),
                ..
            }) => {
                let mut stat = tui_stat.write().unwrap();
                stat.memory_graph_mode = stat.memory_graph_mode.next();
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
            Event::Resize(_, _) => should_update.store(true, Ordering::Relaxed),
            _ => {}
        },
        InputMode::Input(input_event) => {
            if let Event::Key(key) = e {
                match key.code {
                    KeyCode::Enter => {
                        handle_input_value(input_event, tui_stat, data, should_update);
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
    Ok(false)
}

fn handle_input_value(
    input_event: InputEvent,
    tui_stat: &RwLock<TuiStat>,
    data: &RwLock<(Header, ProcList)>,
    should_update: &AtomicBool,
) {
    match input_event {
        InputEvent::NumaNode => {
            let input_value = { tui_stat.read().unwrap().input_value.parse::<usize>() };
            let input_value = match input_value {
                Ok(v) => v,
                Err(_) => {
                    let mut stat = tui_stat.write().unwrap();
                    stat.input_error = Some(" invalid numa node ".into());
                    stat.reset_input();
                    should_update.store(true, Ordering::Relaxed);
                    return;
                }
            };
            let numa_nodes = get_numa_nodes();
            if !numa_nodes.contains_key(&input_value) {
                let mut stat = tui_stat.write().unwrap();
                stat.input_error = Some(" invalid numa node ".into());
                stat.reset_input();
                should_update.store(true, Ordering::Relaxed);
                return;
            }
            let mut stat = tui_stat.write().unwrap();
            stat.cpu_value_mode = CpuValueMode::NumaNode(input_value);
            stat.cpu_column = 1;
            stat.reset_input();
            data.write().unwrap().0.update_cpu(&stat);
            should_update.store(true, Ordering::Relaxed);
        }
    }
}
