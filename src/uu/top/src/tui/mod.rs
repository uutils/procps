// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod color;
mod input;
pub mod stat;

pub use input::*;
use std::borrow::Cow;

use crate::header::{format_memory, Header};
use crate::tui::color::TuiColor;
use crate::tui::stat::{CpuGraphMode, MemoryGraphMode, TuiStat};
use crate::{InfoBar, ProcList};
use ratatui::prelude::*;
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};
use std::cmp::min;

pub struct Tui<'a> {
    settings: &'a crate::Settings,
    header: &'a Header,
    proc_list: &'a ProcList,
    info_bar: &'a Option<InfoBar>,
    stat: &'a mut TuiStat,
}

impl<'a> Tui<'a> {
    pub fn new(
        settings: &'a crate::Settings,
        data: &'a (Header, ProcList, Option<InfoBar>),
        stat: &'a mut TuiStat,
    ) -> Self {
        Self {
            settings,
            header: &data.0,
            proc_list: &data.1,
            info_bar: &data.2,
            stat,
        }
    }

    fn calc_header_height(&self) -> u16 {
        let mut height = u16::from(self.stat.show_load_avg);

        let mut columns = 0;
        if self.stat.cpu_graph_mode != CpuGraphMode::Hide {
            height += 1; // task line
            if self.stat.cpu_graph_mode == CpuGraphMode::Sum {
                height += self.header.cpu.len() as u16;
            } else {
                columns += self.header.cpu.len() as u16;
            }
        }
        if self.stat.memory_graph_mode != MemoryGraphMode::Hide {
            if self.stat.memory_graph_mode == MemoryGraphMode::Sum {
                height += 2;
            } else {
                columns += 2;
            }
        }
        height += columns / self.stat.cpu_column;
        if columns % self.stat.cpu_column != 0 {
            height += 1;
        }

        height
    }

    fn calc_info_bar_height(&self, width: u16) -> u16 {
        if let Some(info_bar) = &self.info_bar {
            let lines: u16 = info_bar
                .content
                .lines()
                .map(|s| (s.len() as u16).div_ceil(width))
                .sum();
            lines + 1 // 1 for title
        } else {
            0
        }
    }

    fn calc_list_coordinates(&self) -> (usize, usize) {
        let list_total = self.proc_list.collected.len();
        let list_offset = self.stat.list_offset;
        (list_offset, list_total)
    }

    fn calc_column_coordinates(&self) -> (usize, usize, usize) {
        let total_columns = self.proc_list.fields.len();
        let horizontal_offset = self.stat.horizontal_offset;
        let column_coordinate = min(horizontal_offset, total_columns - 1);
        let horizontal_offset = if horizontal_offset >= total_columns {
            horizontal_offset - (total_columns - 1)
        } else {
            0
        };
        (column_coordinate, total_columns, horizontal_offset * 8)
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let constraints = vec![Constraint::Length(1); self.calc_header_height() as usize];
        let colorful = self.stat.colorful;

        let cpu = &self.header.cpu;

        let header_layout = Layout::new(Direction::Vertical, constraints).split(area);
        let mut i = 0;

        let mut i_columns = 0;
        let mut cpu_column = None;
        let mut render_bars = |bars_to_render: Vec<(String, f64, f64, f64, f64, char, bool)>,
                               buf: &mut Buffer,
                               i: usize| {
            let mut i = i;
            for (tag, l, r, red, yellow, content, print_percentage) in bars_to_render {
                if cpu_column.is_none() || i_columns >= self.stat.cpu_column as usize {
                    let mut constraints = vec![Constraint::Min(25)];
                    let mut width_left = header_layout[i].width - 25;
                    for _ in 0..self.stat.cpu_column {
                        if width_left > 28 {
                            constraints.extend(vec![Constraint::Length(3), Constraint::Min(25)]);
                            width_left -= 28;
                        } else {
                            constraints.extend(vec![Constraint::Length(0), Constraint::Length(0)]);
                        }
                    }
                    let line =
                        Layout::new(Direction::Horizontal, constraints).split(header_layout[i]);
                    i += 1;
                    i_columns = 0;
                    cpu_column = Some(line);
                }

                let column_offset = i_columns * 2;
                let area = cpu_column.as_ref().unwrap()[column_offset];
                if i_columns > 0 {
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(" ", Style::default().bg_secondary(colorful)),
                        Span::raw(" "),
                    ])
                    .render(cpu_column.as_ref().unwrap()[column_offset - 1], buf);
                }
                let line_layout = Layout::new(
                    Direction::Horizontal,
                    [
                        Constraint::Length(10),
                        Constraint::Length(if self.stat.cpu_column < 3 { 16 } else { 0 }),
                        Constraint::Length(1),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ],
                )
                .split(area);
                i_columns += 1;

                Span::styled(format!("%{tag:<6}:",), Style::default().primary(colorful))
                    .render(line_layout[0], buf);
                let percentage = if print_percentage {
                    format!("{:>5.0}", ((red + yellow) * 100.0).round())
                } else {
                    String::new()
                };
                Line::from(vec![
                    Span::raw(format!("{l:>5.1}")),
                    Span::styled(
                        format!("/{r:<5.1}{percentage}"),
                        Style::default().primary(colorful),
                    ),
                ])
                .render(line_layout[1], buf);
                Paragraph::new("[").render(line_layout[2], buf);

                let width = line_layout[3].width;
                let red_width = (red * width as f64) as u16;
                let yellow_width = (yellow * width as f64) as u16;

                let red_span = Span::styled(
                    content.to_string().repeat(red_width as usize),
                    if content == ' ' {
                        Style::default().bg_primary(colorful)
                    } else {
                        Style::default().primary(colorful)
                    },
                );
                let yellow_span = Span::styled(
                    content.to_string().repeat(yellow_width as usize),
                    if content == ' ' {
                        Style::default().bg_secondary(colorful)
                    } else {
                        Style::default().secondary(colorful)
                    },
                );

                Line::from(vec![red_span, yellow_span]).render(line_layout[3], buf);

                Paragraph::new("]").render(line_layout[4], buf);
            }
            i
        };

        if self.stat.show_load_avg {
            let load_avg = format!(
                "top - {time} {uptime}, {user}, {load_average}",
                time = self.header.uptime.time,
                uptime = self.header.uptime.uptime,
                user = self.header.uptime.user,
                load_average = self.header.uptime.load_average,
            );
            Paragraph::new(load_avg).render(header_layout[i], buf);
            i += 1;
        }

        if self.stat.cpu_graph_mode != CpuGraphMode::Hide {
            let task = &self.header.task;
            let task_line = vec![
                Span::styled("Tasks: ", Style::default().primary(colorful)),
                Span::raw(task.total.to_string()),
                Span::styled(" total, ", Style::default().primary(colorful)),
                Span::raw(task.running.to_string()),
                Span::styled(" running, ", Style::default().primary(colorful)),
                Span::raw(task.sleeping.to_string()),
                Span::styled(" sleeping, ", Style::default().primary(colorful)),
                Span::raw(task.stopped.to_string()),
                Span::styled(" stopped, ", Style::default().primary(colorful)),
                Span::raw(task.zombie.to_string()),
                Span::styled(" zombie", Style::default().primary(colorful)),
            ];
            Line::from(task_line).render(header_layout[i], buf);
            i += 1;

            let mut cpu_bars = Vec::new();
            let bar_content = if self.stat.cpu_graph_mode == CpuGraphMode::Bar {
                '|'
            } else {
                ' '
            };

            for (tag, load) in cpu {
                if self.stat.cpu_graph_mode == CpuGraphMode::Sum {
                    Line::from(vec![
                        Span::styled(format!("%{tag:<6}:  ",), Style::default().red()),
                        Span::raw(format!("{:.1}", load.user)),
                        Span::styled(" us, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.system)),
                        Span::styled(" sy, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.nice)),
                        Span::styled(" ni, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.idle)),
                        Span::styled(" id, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.io_wait)),
                        Span::styled(" wa, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.hardware_interrupt)),
                        Span::styled(" hi, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.software_interrupt)),
                        Span::styled(" si, ", Style::default().red()),
                        Span::raw(format!("{:.1}", load.steal_time)),
                        Span::styled(" st", Style::default().red()),
                    ])
                    .render(header_layout[i], buf);
                    i += 1;

                    continue;
                }

                cpu_bars.push((
                    tag.clone(),
                    load.user,
                    load.system,
                    load.user / 100.0,
                    load.system / 100.0,
                    bar_content,
                    true,
                ));
            }
            i = render_bars(cpu_bars, &mut *buf, i);
        }

        if self.stat.memory_graph_mode != MemoryGraphMode::Hide {
            let mem = &self.header.memory;
            let (unit, unit_name) = match self.settings.scale_summary_mem.as_ref() {
                Some(scale) => match scale.as_str() {
                    "k" => (bytesize::KIB, "KiB"),
                    "m" => (bytesize::MIB, "MiB"),
                    "g" => (bytesize::GIB, "GiB"),
                    "t" => (bytesize::TIB, "TiB"),
                    "p" => (bytesize::PIB, "PiB"),
                    "e" => (1_152_921_504_606_846_976, "EiB"),
                    _ => (bytesize::MIB, "MiB"),
                },
                None => (bytesize::GIB, "GiB"),
            };

            if self.stat.memory_graph_mode == MemoryGraphMode::Sum {
                Line::from(vec![
                    Span::styled(
                        format!("{unit_name} Mem : "),
                        Style::default().primary(colorful),
                    ),
                    Span::raw(format!("{:8.1}", format_memory(mem.total, unit))),
                    Span::styled(" total, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.free, unit))),
                    Span::styled(" free, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.used, unit))),
                    Span::styled(" used, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.buff_cache, unit))),
                    Span::styled(" buff/cache", Style::default().primary(colorful)),
                ])
                .render(header_layout[i], buf);
                i += 1;
                Line::from(vec![
                    Span::styled(
                        format!("{unit_name} Swap: "),
                        Style::default().primary(colorful),
                    ),
                    Span::raw(format!("{:8.1}", format_memory(mem.total_swap, unit))),
                    Span::styled(" total, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.free_swap, unit))),
                    Span::styled(" free, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.used_swap, unit))),
                    Span::styled(" used, ", Style::default().primary(colorful)),
                    Span::raw(format!("{:8.1}", format_memory(mem.available, unit))),
                    Span::styled(" avail Mem", Style::default().primary(colorful)),
                ])
                .render(header_layout[i], buf);
            } else {
                let mut mem_bars = Vec::new();
                let bar_content = if self.stat.memory_graph_mode == MemoryGraphMode::Bar {
                    '|'
                } else {
                    ' '
                };

                mem_bars.push((
                    format!("{unit_name} Mem "), // space to align with "Swap"
                    mem.used as f64 / mem.total as f64 * 100.0,
                    format_memory(mem.total, unit),
                    (mem.total - mem.free - mem.buff_cache) as f64 / mem.total as f64,
                    (mem.free + mem.buff_cache - mem.available) as f64 / mem.total as f64,
                    bar_content,
                    false,
                ));
                if mem.total_swap > 0 {
                    mem_bars.push((
                        format!("{unit_name} Swap"),
                        mem.used_swap as f64 / mem.total_swap as f64 * 100.0,
                        format_memory(mem.total_swap, unit),
                        0.0,
                        mem.used_swap as f64 / mem.total_swap as f64,
                        bar_content,
                        false,
                    ));
                } else {
                    mem_bars.push((
                        format!("{unit_name} Swap"),
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        bar_content,
                        false,
                    ));
                }
                render_bars(mem_bars, &mut *buf, i);
            }
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let colorful = self.stat.colorful;
        if let Some(v) = self.stat.input_message.as_ref() {
            let layout = Layout::new(
                Direction::Horizontal,
                [Constraint::Length(v.len() as u16), Constraint::Fill(1)],
            )
            .split(area);
            Paragraph::new(v.as_str())
                .style(Style::default().error(colorful))
                .render(layout[0], buf);
            return;
        }
        let input = if !self.stat.input_label.is_empty() || !self.stat.input_value.is_empty() {
            Line::from(vec![
                Span::styled(&self.stat.input_label, Style::default().primary(colorful)),
                Span::raw(" "),
                Span::raw(&self.stat.input_value),
            ])
        } else if self.stat.show_coordinates {
            let list_coordinates = self.calc_list_coordinates();
            let column_coordinates = self.calc_column_coordinates();
            Line::from(vec![
                Span::raw(format!(
                    "  scroll coordinates: y = {}/{} (tasks), x = {}/{} (fields)",
                    list_coordinates.0 + 1,
                    list_coordinates.1,
                    column_coordinates.0 + 1,
                    column_coordinates.1
                )),
                Span::raw(if column_coordinates.2 > 0 {
                    format!(" + {}", column_coordinates.2)
                } else {
                    String::new()
                }),
            ])
        } else {
            Line::from("")
        };
        input.render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let colorful = self.stat.colorful;
        let highlight_sorted = self.stat.highlight_sorted;
        let highlight_bold = self.stat.highlight_bold;
        let sorter = if self.stat.sort_by_pid {
            "PID"
        } else {
            &self.stat.sorter
        };
        let highlight_column = self
            .proc_list
            .fields
            .iter()
            .position(|f| f == sorter)
            .unwrap_or(0);
        let user_width = {
            if let Some(width) = self.stat.width_increment {
                10 + width
            } else if let Some(user_column_nth) =
                self.proc_list.fields.iter().position(|f| f == "USER")
            {
                let users: Vec<&String> = self
                    .proc_list
                    .collected
                    .iter()
                    .map(|item| &item.1[user_column_nth])
                    .collect();
                users.iter().map(|u| u.len()).max().unwrap_or_default() + 1
            } else {
                10
            }
        };
        let build_constraint = |field: &str| match field {
            "PID" => Constraint::Length(7),
            "USER" => Constraint::Length(user_width as u16),
            "PR" => Constraint::Length(4),
            "NI" => Constraint::Length(4),
            "VIRT" => Constraint::Length(8),
            "RES" => Constraint::Length(8),
            "SHR" => Constraint::Length(8),
            "S" => Constraint::Length(3),
            "%CPU" => Constraint::Length(6),
            "%MEM" => Constraint::Length(6),
            "TIME+" => Constraint::Length(10),
            "COMMAND" => Constraint::Min(20),
            _ => Constraint::Length(0),
        };

        let list_coordinates = self.calc_list_coordinates();
        let column_coordinates = self.calc_column_coordinates();

        let constraints: Vec<Constraint> = self
            .proc_list
            .fields
            .iter()
            .map(|field| build_constraint(field))
            .skip(column_coordinates.0)
            .collect();

        let header = Row::new(
            self.proc_list
                .fields
                .clone()
                .split_off(column_coordinates.0),
        )
        .style(Style::default().bg_secondary(colorful));

        let rows = self.proc_list.collected.iter().map(|item| {
            let cells = item
                .1
                .iter()
                .enumerate()
                .skip(column_coordinates.0)
                .map(|(n, c)| {
                    let c = if column_coordinates.2 > 0 {
                        if c.len() < column_coordinates.2 {
                            // handle offset
                            Cow::Borrowed("")
                        } else {
                            Cow::Borrowed(&c[column_coordinates.2..])
                        }
                    } else if let Constraint::Length(length) = &constraints[n] {
                        // truncate if too long
                        if c.len() > *length as usize {
                            Cow::Owned(format!("{}+", &c[0..*length as usize - 2]))
                        } else {
                            Cow::Borrowed(c.as_str())
                        }
                    } else {
                        Cow::Borrowed(c.as_str())
                    };
                    if highlight_sorted && n == highlight_column {
                        Cell::from(Span::styled(
                            c,
                            if highlight_bold {
                                Style::default().bg_primary(colorful)
                            } else {
                                Style::default().primary(colorful)
                            },
                        ))
                    } else {
                        Cell::from(c)
                    }
                });
            Row::new(cells).height(1)
        });

        let mut state = TableState::default().with_offset(list_coordinates.0);

        let table = Table::new(rows, constraints.clone()).header(header);
        StatefulWidget::render(table, area, buf, &mut state);
    }

    fn render_info_bar(&self, area: Rect, buf: &mut Buffer) {
        if let Some(info_bar) = self.info_bar.as_ref() {
            let constraints = [Constraint::Length(1), Constraint::Min(1)];
            let layout = Layout::new(Direction::Vertical, constraints).split(area);
            Line::from(Span::styled(
                format!("{:<width$}", &info_bar.title, width = area.width as usize),
                Style::default().bg_secondary(self.stat.colorful),
            ))
            .render(layout[0], buf);
            let mut lines = vec![];
            let width = layout[1].width as usize;
            info_bar.content.lines().for_each(|s| {
                let mut start = 0;
                let len = s.len();
                while start < len {
                    let end = (start + width).min(len);
                    lines.push(Line::from(&s[start..end]));
                    start = end;
                }
            });
            Paragraph::new(lines).render(layout[1], buf);
        }
    }
}

impl Widget for Tui<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.stat.list_offset = min(
            self.stat.list_offset,
            self.proc_list
                .collected
                .len()
                .checked_sub(1)
                .unwrap_or_default(),
        );
        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(self.calc_header_height()),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(self.calc_info_bar_height(area.width)),
            ],
        )
        .split(area);

        self.render_header(layout[0], buf);
        self.render_input(layout[1], buf);
        let mut list_area = layout[2];
        if self.stat.max_list_display > 0 {
            let list_height = min(layout[2].height, self.stat.max_list_display as u16) + 1; // 1 for header
            list_area.height = list_height;
        }
        self.render_list(list_area, buf);
        self.render_info_bar(layout[3], buf);
    }
}
