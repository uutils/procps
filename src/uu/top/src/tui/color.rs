// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use ratatui::style::{Color, Style};

/// This is the trait used to adjust the TUI color options
pub(crate) trait TuiColorHelper: Sized {
    fn primary(self, colorful: bool) -> Style;
    fn bg_primary(self, colorful: bool) -> Style;
    fn secondary(self, colorful: bool) -> Style;
    fn bg_secondary(self, colorful: bool) -> Style;
    fn error(self, colorful: bool) -> Style;
}

impl TuiColorHelper for Style {
    fn primary(self, colorful: bool) -> Style {
        if colorful {
            self.red()
        } else {
            self
        }
    }

    fn bg_primary(self, colorful: bool) -> Style {
        let style = self.fg(Color::Black);
        if colorful {
            style.bg(Color::Red)
        } else {
            style.bg(Color::White)
        }
    }

    fn secondary(self, colorful: bool) -> Style {
        if colorful {
            self.yellow()
        } else {
            self
        }
    }

    fn bg_secondary(self, colorful: bool) -> Style {
        let style = self.fg(Color::Black);
        if colorful {
            style.bg(Color::Yellow)
        } else {
            style.bg(Color::White)
        }
    }

    fn error(self, colorful: bool) -> Style {
        if colorful {
            self.fg(Color::Black).bg(Color::Red)
        } else {
            self
        }
    }
}
