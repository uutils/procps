use ratatui::prelude::Stylize;
use ratatui::style::{Color, Styled};

pub(crate) trait TuiColor<'a, T>: Sized {
    fn primary(self, colorful: bool) -> T;
    fn bg_primary(self, colorful: bool) -> T;
    fn secondary(self, colorful: bool) -> T;
    fn bg_secondary(self, colorful: bool) -> T;
    fn error(self, colorful: bool) -> T;
}

impl<'a, T, U> TuiColor<'a, T> for U
where
    U: Styled<Item = T>,
{
    fn primary(self, colorful: bool) -> T {
        let style = self.style();
        if colorful {
            self.red()
        } else {
            self.set_style(style)
        }
    }

    fn bg_primary(self, colorful: bool) -> T {
        let style = self.style().fg(Color::Black);
        if colorful {
            self.set_style(style.bg(Color::Red))
        } else {
            self.set_style(style.bg(Color::White))
        }
    }

    fn secondary(self, colorful: bool) -> T {
        let style = self.style();
        if colorful {
            self.yellow()
        } else {
            self.set_style(style)
        }
    }

    fn bg_secondary(self, colorful: bool) -> T {
        let style = self.style().fg(Color::Black);
        if colorful {
            self.set_style(style.bg(Color::Yellow))
        } else {
            self.set_style(style.bg(Color::White))
        }
    }

    fn error(self, colorful: bool) -> T {
        let style = self.style();
        if colorful {
            self.set_style(style.fg(Color::Black).bg(Color::Red))
        } else {
            self.set_style(style)
        }
    }
}
