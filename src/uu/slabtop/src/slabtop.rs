// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub use crate::parse::SlabInfo;
use clap::{arg, crate_version, ArgAction, Command};
use std::fmt::Write;
use uucore::error::UResult;

mod parse;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let sort_flag = matches
        .try_get_one::<char>("sort")
        .ok()
        .unwrap_or(Some(&'o'))
        .unwrap_or(&'o');

    let human = matches.get_flag("human");

    let slabinfo = SlabInfo::new()?.sort(*sort_flag, false);

    if matches.get_flag("once") {
        println!("{}", build_header(&slabinfo, human));
        print!("{}", build_list(&slabinfo, human)); // new line included
    } else {
        // TODO: implement TUI
        println!("{}", build_header(&slabinfo, human));
        print!("{}", build_list(&slabinfo, human)); // new line included
    }

    Ok(())
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

// This is a copy from uu_free::humanized with minor modifications.
// TODO: Extract this function to shared crate in future
fn humanized(b: u64, si: bool) -> String {
    let units = ['B', 'K', 'M', 'G', 'T', 'P'];
    let mut level = 0;
    let mut divisor = 1;
    while level < units.len() - 1 && divisor * 1000 <= b {
        divisor *= if si { 1000 } else { 1024 };
        level += 1;
    }
    if level == 0 {
        return format!("{}{}", b, units[level]);
    }

    let value = (b as f64) / (divisor as f64);
    let formatted_value = if (value * 10.0).round() < 100.0 {
        format!("{:.1}", (value * 10.0).round() / 10.0)
    } else {
        (value as u64).to_string()
    };

    format!(
        "{}{}{}",
        formatted_value,
        units[level].to_owned(),
        if si { "" } else { "i" }
    )
}

fn build_header(slabinfo: &SlabInfo, human: bool) -> String {
    let mut out = String::new();

    writeln!(
        &mut out,
        r" Active / Total Objects (% used)    : {} / {} ({:.1}%)",
        slabinfo.total_active_objs(),
        slabinfo.total_objs(),
        percentage(slabinfo.total_active_objs(), slabinfo.total_objs())
    )
    .unwrap();

    writeln!(
        &mut out,
        r" Active / Total Slabs (% used)      : {} / {} ({:.1}%)",
        slabinfo.total_active_slabs(),
        slabinfo.total_slabs(),
        percentage(slabinfo.total_active_slabs(), slabinfo.total_slabs(),)
    )
    .unwrap();

    // TODO: I don't know the 'cache' meaning.
    writeln!(
        &mut out,
        r" Active / Total Caches (% used)     : {} / {} ({:.1}%)",
        slabinfo.total_active_cache(),
        slabinfo.total_cache(),
        percentage(slabinfo.total_active_cache(), slabinfo.total_cache())
    )
    .unwrap();

    let format_size = |size: u64| -> String {
        if human {
            humanized(size, false)
        } else {
            (size / 1024).to_string() // implicit unit 'K', to match the output of GNU
        }
    };
    writeln!(
        &mut out,
        r" Active / Total Size (% used)       : {} / {} ({:.1}%)",
        format_size(slabinfo.total_active_size()),
        format_size(slabinfo.total_size()),
        percentage(slabinfo.total_active_size(), slabinfo.total_size())
    )
    .unwrap();

    writeln!(
        &mut out,
        r" Minimum / Average / Maximum Object : {:.2}K / {:.2}K / {:.2}K",
        to_kb(slabinfo.object_minimum()),
        to_kb(slabinfo.object_avg()),
        to_kb(slabinfo.object_maximum())
    )
    .unwrap();

    out
}

fn build_list(info: &SlabInfo, human: bool) -> String {
    let mut out = String::new();
    let title = format!(
        "{:>6} {:>6} {:>4} {:>8} {:>6} {:>8} {:>10} {:<}",
        "OBJS", "ACTIVE", "USE", "OBJ SIZE", "SLABS", "OBJ/SLAB", "CACHE SIZE", "NAME"
    );
    writeln!(&mut out, "{title}").unwrap();

    for name in info.names() {
        let objs = info.fetch(name, "num_objs").unwrap_or_default();
        let active = info.fetch(name, "active_objs").unwrap_or_default();
        let used = format!("{:.0}%", percentage(active, objs));
        let objsize = {
            let size = info.fetch(name, "objsize").unwrap_or_default(); // Byte to KB :1024
            size as f64 / 1024.0
        };
        let slabs = info.fetch(name, "num_slabs").unwrap_or_default();
        let obj_per_slab = info.fetch(name, "objperslab").unwrap_or_default();

        let cache_size = (objsize * (objs as f64)) as u64;
        let cache_size_str = if human {
            humanized(cache_size, false)
        } else {
            cache_size.to_string()
        };

        let objsize = format!("{objsize:.2}");

        let content = format!(
            "{objs:>6} {active:>6} {used:>4} {objsize:>7}K {slabs:>6} {obj_per_slab:>8} {cache_size_str:>10} {name:<}"
        );

        writeln!(&mut out, "{content}").unwrap();
    }
    out
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    const AFTER_HELP: &str = "The following are valid sort criteria:

 a    sort by number of active objects
 b    sort by objects per slab
 c    sort by cache size
 l    sort by number of slabs
 v    sort by (non display) number of active slabs
 n    sort by name
 o    sort by number of objects (the default)
 p    sort by (non display) pages per slab
 s    sort by object size
 u    sort by cache utilization";

    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Display kernel slab cache information in real time")
        .override_usage("slabtop [options]")
        .infer_long_args(true)
        .args([
            // arg!(-d --delay <secs>  "delay updates"),
            arg!(-o --once          "only display once, then exit").action(ArgAction::SetTrue),
            arg!(--human            "show human-readable output").action(ArgAction::SetTrue),
            arg!(-s --sort  <char>  "specify sort criteria by character (see below)"),
        ])
        .after_help(AFTER_HELP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_format() {
        let test_data = include_str!("../../../../tests/fixtures/slabtop/data.txt");
        let slabinfo = SlabInfo::parse(test_data).unwrap();
        let header = build_header(&slabinfo, false);
        let list = build_list(&slabinfo, false);

        assert!(header.contains("2612866 / 2661422 (98.2%)")); // Objects
        assert!(header.contains("54986 / 54986 (100.0%)")); // Slabs
        assert!(header.contains("520232232 / 533968024 (97.4%)")); // Caches
        assert!(header.contains("508039 / 521453 (97.4%)")); // Size
        assert!(header.contains("0.01K / 0.95K / 8.00K")); // Object

        assert!(
            list.contains("0      0   0%    0.20K      0       39          0 nf_conntrack_expect")
        ); // 0
        assert!(list.contains("78      2   3%    0.81K      2       39         63 QIPCRTR")); // B
        assert!(list
            .contains("13906   7805  56%    0.23K    409       34       3259 btrfs_extent_buffer"));
        // Ki
    }

    #[test]
    fn test_data_format_human() {
        let test_data = include_str!("../../../../tests/fixtures/slabtop/data.txt");
        let slabinfo = SlabInfo::parse(test_data).unwrap();
        let header = build_header(&slabinfo, true);
        let list = build_list(&slabinfo, true);

        // '*' for humanizing applied lines
        assert!(header.contains("2612866 / 2661422 (98.2%)")); // Objects
        assert!(header.contains("54986 / 54986 (100.0%)")); // Slabs
        assert!(header.contains("520232232 / 533968024 (97.4%)")); // Caches
        assert!(header.contains("496Mi / 509Mi (97.4%)")); // Size*
        assert!(header.contains("0.01K / 0.95K / 8.00K")); // Object

        assert!(
            list.contains("0      0   0%    0.20K      0       39         0B nf_conntrack_expect")
        ); // 0*
        assert!(list.contains("78      2   3%    0.81K      2       39        63B QIPCRTR")); // B*
        assert!(list
            .contains("13906   7805  56%    0.23K    409       34      3.2Ki btrfs_extent_buffer"));
        // Ki*
    }
}
