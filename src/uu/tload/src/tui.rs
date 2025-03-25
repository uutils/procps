// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::Marker,
    text::{Line, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};

use crate::SystemLoadAvg;

pub(crate) struct ModernTui<'a>(&'a [SystemLoadAvg]);

impl ModernTui<'_> {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![
            Line::from(format!(
                "Last 1 min load:   {:>5}",
                self.0.last().unwrap().last_1
            )),
            Line::from(format!(
                "Last 5 min load:   {:>5}",
                self.0.last().unwrap().last_5
            )),
            Line::from(format!(
                "Last 10 min load:  {:>5}",
                self.0.last().unwrap().last_10
            )),
        ]);

        Paragraph::new(text)
            .style(Style::default().bold().italic())
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("System load history"),
            )
            .render(area, buf);
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        let result = &self.0[self.0.len().saturating_sub(area.width.into())..]
            .iter()
            .enumerate()
            .map(|(index, load)| (index as f64, load.last_1 as f64))
            .collect::<Vec<_>>();

        let data = Dataset::default()
            .graph_type(GraphType::Line)
            .marker(Marker::Braille)
            .data(result);

        let x_axis = {
            let start = Line::from("0");
            let middle = Line::from((area.width / 2).to_string());
            let end = Line::from(area.width.to_string());
            Axis::default()
                .title("Time(per delay)")
                .bounds([0.0, area.width.into()])
                .labels(vec![start, middle, end])
        };

        // Why this tweak?
        //
        // Sometime the chart cannot display all the line because of max height are equals the max
        // load of system in the history, so I add 0.2*{max_load} to the height of chart make it
        // display beautiful
        let y_axis_upper_bound = result.iter().map(|it| it.1).reduce(f64::max).unwrap_or(0.0);
        let y_axis_upper_bound = y_axis_upper_bound + y_axis_upper_bound * 0.2;
        let label = {
            let min = "0.0".to_owned();
            let mid = format!("{:.1}", y_axis_upper_bound / 2.0);
            let max = format!("{:.1}", y_axis_upper_bound);
            vec![min, mid, max]
        };
        let y_axis = Axis::default()
            .bounds([0.0, y_axis_upper_bound])
            .labels(label)
            .title("System Load");

        Chart::new(vec![data])
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(area, buf);
    }
}

impl ModernTui<'_> {
    pub(crate) fn new(input: &[SystemLoadAvg]) -> ModernTui<'_> {
        ModernTui(input)
    }
}

impl Widget for ModernTui<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(5), Constraint::Min(0)],
        )
        .split(area);

        let header = layout[0];
        let chart = layout[1];

        self.render_header(header, buf);
        self.render_chart(chart, buf);
    }
}

// TODO: Implemented LegacyTui
pub(crate) type LegacyTui<'a> = ModernTui<'a>;
