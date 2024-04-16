// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::parse::SlabInfo;
use clap::{arg, crate_version, ArgAction, Command};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("slabtop.md");
const USAGE: &str = help_usage!("slabtop.md");

mod parse;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let sort_flag = matches
        .try_get_one::<char>("sort")
        .ok()
        .unwrap_or(Some(&'o'))
        .unwrap_or(&'o');

    let slabinfo = SlabInfo::new()?.sort(*sort_flag, false);

    println!(
        r" Active / Total Objects (% used)    : {} / {} ({:.1}%)",
        slabinfo.total_active_objs(),
        slabinfo.total_objs(),
        percentage(slabinfo.total_active_objs(), slabinfo.total_objs())
    );
    println!(
        r" Active / Total Slabs (% used)      : {} / {} ({:.1}%)",
        slabinfo.total_active_slabs(),
        slabinfo.total_slabs(),
        percentage(slabinfo.total_active_slabs(), slabinfo.total_slabs(),)
    );

    // TODO: I don't know the 'cache' meaning.
    println!(
        r" Active / Total Caches (% used)     : {} / {} ({:.1}%)",
        slabinfo.total_active_cache(),
        slabinfo.total_cache(),
        percentage(slabinfo.total_active_cache(), slabinfo.total_cache())
    );

    println!(
        r" Active / Total Size (% used)       : {:.2}K / {:.2}K ({:.1}%)",
        to_kb(slabinfo.total_active_size()),
        to_kb(slabinfo.total_size()),
        percentage(slabinfo.total_active_size(), slabinfo.total_size())
    );

    println!(
        r" Minimum / Average / Maximum Object : {:.2}K / {:.2}K / {:.2}K",
        to_kb(slabinfo.object_minimum()),
        to_kb(slabinfo.object_avg()),
        to_kb(slabinfo.object_maximum())
    );

    // separate header info and slab list
    println!();

    // TODO: TUI Implementation
    let title = format!(
        "{:>6} {:>6} {:>4} {:>8} {:>6} {:>8} {:>10} {:<}",
        "OBJS", "ACTIVE", "USE", "OBJ SIZE", "SLABS", "OBJ/SLAB", "CACHE SIZE", "NAME"
    );
    println!("{}", title);

    output(&slabinfo);

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

fn output(info: &SlabInfo) {
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
        let objsize = format!("{:.2}", objsize);

        let content = format!(
            "{:>6} {:>6} {:>4} {:>7}K {:>6} {:>8} {:>10} {:<}",
            objs, active, used, objsize, slabs, obj_per_slab, cache_size, name
        );

        println!("{}", content);
    }
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .args([
            // arg!(-d --delay <secs>  "delay updates"),
            // arg!(-o --once          "only display once, then exit"),
            arg!(-s --sort  <char>  "specify sort criteria by character (see below)"),
            arg!(-h --help          "display this help and exit").action(ArgAction::Help),
        ])
        .after_help(
            r"The following are valid sort criteria:
 a: sort by number of active objects
 b: sort by objects per slab
 c: sort by cache size
 l: sort by number of slabs
 v: sort by (non display) number of active slabs
 n: sort by name
 o: sort by number of objects (the default)
 p: sort by (non display) pages per slab
 s: sort by object size
 u: sort by cache utilization",
        )
}
