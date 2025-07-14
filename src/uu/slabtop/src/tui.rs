// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use ratatui::{
    prelude::*,
    widgets::{List, ListItem, Widget},
};

use crate::SlabInfo;

pub(crate) struct Tui<'a> {
    slabinfo: &'a SlabInfo,
}

impl Tui<'_> {
    pub(crate) fn new(slabinfo: &'_ SlabInfo) -> Tui<'_> {
        Tui { slabinfo }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            format!(
                r" Active / Total Objects (% used)    : {} / {} ({:.1}%)",
                self.slabinfo.total_active_objs(),
                self.slabinfo.total_objs(),
                percentage(
                    self.slabinfo.total_active_objs(),
                    self.slabinfo.total_objs()
                )
            ),
            format!(
                r" Active / Total Slabs (% used)      : {} / {} ({:.1}%)",
                self.slabinfo.total_active_slabs(),
                self.slabinfo.total_slabs(),
                percentage(
                    self.slabinfo.total_active_slabs(),
                    self.slabinfo.total_slabs(),
                )
            ),
            // TODO: I don't know the 'cache' meaning.
            format!(
                r" Active / Total Caches (% used)     : {} / {} ({:.1}%)",
                self.slabinfo.total_active_cache(),
                self.slabinfo.total_cache(),
                percentage(
                    self.slabinfo.total_active_cache(),
                    self.slabinfo.total_cache()
                )
            ),
            format!(
                r" Active / Total Size (% used)       : {:.2}K / {:.2}K ({:.1}%)",
                to_kb(self.slabinfo.total_active_size()),
                to_kb(self.slabinfo.total_size()),
                percentage(
                    self.slabinfo.total_active_size(),
                    self.slabinfo.total_size()
                )
            ),
            format!(
                r" Minimum / Average / Maximum Object : {:.2}K / {:.2}K / {:.2}K",
                to_kb(self.slabinfo.object_minimum()),
                to_kb(self.slabinfo.object_avg()),
                to_kb(self.slabinfo.object_maximum())
            ),
        ]
        .into_iter()
        .map(Line::from);

        Widget::render(List::new(lines), area, buf);
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let mut list = vec![ListItem::from(format!(
            "{:>6} {:>6} {:>4} {:>8} {:>6} {:>8} {:>10} {:<}",
            "OBJS", "ACTIVE", "USE", "OBJ SIZE", "SLABS", "OBJ/SLAB", "CACHE SIZE", "NAME"
        ))
        .bg(Color::Black)];

        self.slabinfo.names().truncate(area.height.into());
        list.extend(
            self.slabinfo
                .names()
                .iter()
                .map(|name| self.build_list_item(name)),
        );

        Widget::render(List::new(list), area, buf);
    }

    fn build_list_item(&self, name: &str) -> ListItem<'_> {
        let objs = self.slabinfo.fetch(name, "num_objs").unwrap_or_default();
        let active = self.slabinfo.fetch(name, "active_objs").unwrap_or_default();
        let used = format!("{:.0}%", percentage(active, objs));
        let objsize = {
            let size = self.slabinfo.fetch(name, "objsize").unwrap_or_default(); // Byte to KB :1024
            size as f64 / 1024.0
        };
        let slabs = self.slabinfo.fetch(name, "num_slabs").unwrap_or_default();
        let obj_per_slab = self.slabinfo.fetch(name, "objperslab").unwrap_or_default();

        let cache_size = (objsize * (objs as f64)) as u64;
        let objsize = format!("{objsize:.2}");

        ListItem::from(format!(
            "{objs:>6} {active:>6} {used:>4} {objsize:>7}K {slabs:>6} {obj_per_slab:>8} {cache_size:>10} {name:<}"
        ))
    }
}

impl Widget for Tui<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // layout[0]: Header
        // layout[1]: List of process
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Max(6), Constraint::Min(0)],
        )
        .split(area);

        let header = layout[0];
        let list = layout[1];

        self.render_header(header, buf);
        self.render_list(list, buf);
    }
}

fn to_kb(byte: u64) -> f64 {
    byte as f64 / 1024.0
}

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    let numerator = numerator as f64;
    let denominator = denominator as f64;

    (numerator / denominator) * 100.0
}
