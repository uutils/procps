use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Paragraph, Row, Table},
    Terminal
};
use std::io::{Result, stdout};

pub fn start_tui<F>(data_provider: F) -> Result<()>
where
    F: Fn() -> (Vec<String>, Vec<Vec<String>>),
{
    enable_raw_mode()?;
    
    let stdout = stdout();
        
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, data_provider);

    disable_raw_mode()?;
    
    terminal.show_cursor()?;

    result
}

fn render_top_info() -> Vec<String> {
    let info = "top - 04:30:13 up  6:02,  3 users,  load average: 0.40, 0.89, 1.24
Tasks: 260 total,   3 running, 257 sleeping,   0 stopped,   0 zombie
%Cpu(s):  1.9 us,  0.3 sy,  0.0 ni, 97.8 id,  0.0 wa,  0.0 hi,  0.0 si,  0.0 st
MiB Mem :   3911.6 total,    190.6 free,   2549.1 used,   1171.9 buff/cache
MiB Swap:   2048.0 total,    692.0 free,   1356.0 used.   1017.7 avail Mem";
    
    info.lines().map(String::from).collect()
}

fn run_app<F, B>(terminal: &mut Terminal<B>, data_provider: F) -> Result<()>
where
    F: Fn() -> (Vec<String>, Vec<Vec<String>>),
    B: Backend,
{
    loop {
        let (fields, data) = data_provider();

        terminal.clear().unwrap();
        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(1)].as_ref())
                .spacing(1)
                .split(f.size());

            let rows = data.into_iter().map(|row| Row::new(row));
            let widths = (0..fields.len())
                .map(|_| Constraint::Length(10))
                .collect::<Vec<_>>();
            
            let top_paragraph = Paragraph::new(Text::from(
                render_top_info()
                    .into_iter()
                    .map(|line| Line::from(line))
                    .collect::<Vec<_>>()
            ));

            let table = Table::new(rows, widths)
                .header(Row::new(fields).style(Style::default().fg(Color::Black).bg(Color::White)))
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD),
                );

            f.render_widget(top_paragraph, layout[0]);
            f.render_widget(table, layout[1]);
        })?;

        // handle events
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match key.code {
                KeyCode::Char('q') => {
                    println!();
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
