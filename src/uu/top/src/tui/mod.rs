// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod stat;

use crate::header::{format_memory, Header};
use crate::tui::stat::{CpuGraphMode, MemoryGraphMode, TuiStat};
use crate::ProcList;
use ratatui::prelude::*;
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};
use std::cmp::min;

pub struct Tui<'a> {
    settings: &'a crate::Settings,
    header: &'a Header,
    proc_list: &'a ProcList,
    stat: &'a mut TuiStat,
}

impl<'a> Tui<'a> {
    pub fn new(
        settings: &'a crate::Settings,
        data: &'a (Header, ProcList),
        stat: &'a mut TuiStat,
    ) -> Self {
        Self {
            settings,
            header: &data.0,
            proc_list: &data.1,
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

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let constraints = vec![Constraint::Length(1); self.calc_header_height() as usize];

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
                        Span::styled(" ", Style::default().bg(Color::Yellow)),
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

                Span::styled(format!("%{tag:<6}:",), Style::default().red())
                    .render(line_layout[0], buf);
                let percentage = if print_percentage {
                    format!("{:>5.0}", ((red + yellow) * 100.0).round())
                } else {
                    String::new()
                };
                Line::from(vec![
                    Span::raw(format!("{l:>5.1}")),
                    Span::styled(format!("/{r:<5.1}{percentage}"), Style::default().red()),
                ])
                .render(line_layout[1], buf);
                Paragraph::new("[").render(line_layout[2], buf);

                let width = line_layout[3].width;
                let red_width = (red * width as f64) as u16;
                let yellow_width = (yellow * width as f64) as u16;

                let red_span = Span::styled(
                    content.to_string().repeat(red_width as usize),
                    if content == ' ' {
                        Style::default().bg(Color::Red)
                    } else {
                        Style::default().red()
                    },
                );
                let yellow_span = Span::styled(
                    content.to_string().repeat(yellow_width as usize),
                    if content == ' ' {
                        Style::default().bg(Color::Yellow)
                    } else {
                        Style::default().yellow()
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
                Span::styled("Tasks: ", Style::default().red()),
                Span::raw(task.total.to_string()),
                Span::styled(" total, ", Style::default().red()),
                Span::raw(task.running.to_string()),
                Span::styled(" running, ", Style::default().red()),
                Span::raw(task.sleeping.to_string()),
                Span::styled(" sleeping, ", Style::default().red()),
                Span::raw(task.stopped.to_string()),
                Span::styled(" stopped, ", Style::default().red()),
                Span::raw(task.zombie.to_string()),
                Span::styled(" zombie", Style::default().red()),
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
                    Span::styled(format!("{unit_name} Mem : "), Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.total, unit))),
                    Span::styled(" total, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.free, unit))),
                    Span::styled(" free, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.used, unit))),
                    Span::styled(" used, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.buff_cache, unit))),
                    Span::styled(" buff/cache", Style::default().red()),
                ])
                .render(header_layout[i], buf);
                i += 1;
                Line::from(vec![
                    Span::styled(format!("{unit_name} Swap: "), Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.total_swap, unit))),
                    Span::styled(" total, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.free_swap, unit))),
                    Span::styled(" free, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.used_swap, unit))),
                    Span::styled(" used, ", Style::default().red()),
                    Span::raw(format!("{:8.1}", format_memory(mem.available, unit))),
                    Span::styled(" avail Mem", Style::default().red()),
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
        let input = Paragraph::new("");
        input.render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let build_constraint = |field: &str| match field {
            "PID" => Constraint::Length(7),
            "USER" => Constraint::Length(10),
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

        let constraints: Vec<Constraint> = self
            .proc_list
            .fields
            .iter()
            .map(|field| build_constraint(field))
            .collect();

        self.stat.list_offset = min(self.stat.list_offset, self.proc_list.collected.len() - 1);

        let header = Row::new(self.proc_list.fields.clone())
            .style(Style::default().bg(Color::Yellow))
            .height(1);

        let rows = self.proc_list.collected.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(1)
        });

        let mut state = TableState::default().with_offset(self.stat.list_offset);

        let table = Table::new(rows, constraints).header(header);
        StatefulWidget::render(table, area, buf, &mut state);
    }
}

impl Widget for Tui<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(self.calc_header_height()),
                Constraint::Length(1),
                Constraint::Min(0),
            ],
        )
        .split(area);

        self.render_header(layout[0], buf);
        self.render_input(layout[1], buf);
        self.render_list(layout[2], buf);
    }
}
