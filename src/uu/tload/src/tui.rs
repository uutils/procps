use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::Marker,
    text::{Line, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};

use crate::SystemLoadAvg;

pub(crate) struct ModernTui<'a>(&'a [SystemLoadAvg]);

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
        // Header
        {
            let area = layout[0];
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
        };

        // Chart
        {
            let area = layout[1];

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
            let y_axis = Axis::default().bounds([0.0, 10.0]).title("System Load");

            Chart::new(vec![data])
                .x_axis(x_axis)
                .y_axis(y_axis)
                .render(area, buf);
        };
    }
}

// TODO: Implemented LegacyTui
pub(crate) type LegacyTui<'a> = ModernTui<'a>;
