// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod parser;
mod picker;

#[cfg(target_os = "linux")]
use crate::picker::{get_disk_sum, get_pickers, get_stats, Picker};
use clap::value_parser;
#[allow(unused_imports)]
use clap::{arg, crate_version, ArgMatches, Command};
#[allow(unused_imports)]
pub use parser::*;
#[allow(unused_imports)]
use uucore::error::{UResult, USimpleError};

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    #[allow(unused)]
    let matches = uu_app().try_get_matches_from(args)?;
    #[cfg(target_os = "linux")]
    {
        let wide = matches.get_flag("wide");
        let one_header = matches.get_flag("one-header");
        let no_first = matches.get_flag("no-first");
        let term_height = terminal_size::terminal_size()
            .map(|size| size.1 .0)
            .unwrap_or(0);

        if matches.get_flag("forks") {
            return print_forks();
        }
        if matches.get_flag("slabs") {
            return print_slabs(one_header, term_height);
        }
        if matches.get_flag("stats") {
            return print_stats();
        }
        if matches.get_flag("disk") {
            return print_disk(wide, one_header, term_height);
        }
        if matches.get_flag("disk-sum") {
            return print_disk_sum();
        }
        if let Some(device) = matches.get_one::<String>("partition") {
            return print_partition(device);
        }

        // validate unit
        if let Some(unit) = matches.get_one::<String>("unit") {
            if !["k", "K", "m", "M"].contains(&unit.as_str()) {
                Err(USimpleError::new(
                    1,
                    "-S requires k, K, m or M (default is KiB)",
                ))?;
            }
        }

        let delay = matches.get_one::<u64>("delay");
        let count = matches.get_one::<u64>("count");
        let mut count = count.copied().map(|c| if c == 0 { 1 } else { c });
        let delay = delay.copied().unwrap_or_else(|| {
            count.get_or_insert(1);
            1
        });

        let pickers = get_pickers(&matches);
        let mut proc_data = ProcData::new();

        let mut line_count = 0;
        print_header(&pickers);
        if !no_first {
            print_data(&pickers, &proc_data, None, &matches);
            line_count += 1;
        }

        while count.is_none() || line_count < count.unwrap() {
            std::thread::sleep(std::time::Duration::from_secs(delay));
            let proc_data_now = ProcData::new();
            if needs_header(one_header, term_height, line_count) {
                print_header(&pickers);
            }
            print_data(&pickers, &proc_data_now, Some(&proc_data), &matches);
            line_count += 1;
            proc_data = proc_data_now;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_forks() -> UResult<()> {
    let data = get_stats();

    let fork_data = data.last().unwrap();
    println!("{:>13} {}", fork_data.1, fork_data.0);

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_stats() -> UResult<()> {
    let data = get_stats();

    data.iter()
        .for_each(|(name, value)| println!("{value:>13} {name}"));

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_slabs(one_header: bool, term_height: u16) -> UResult<()> {
    let mut slab_data = uu_slabtop::SlabInfo::new()?.data;

    slab_data.sort_by_key(|k| k.0.to_lowercase());

    print_slab_header();

    for (line_count, slab_item) in slab_data.into_iter().enumerate() {
        if needs_header(one_header, term_height, line_count as u64) {
            print_slab_header();
        }

        println!(
            "{:<24} {:>6} {:>6} {:>6} {:>6}",
            slab_item.0, slab_item.1[0], slab_item.1[1], slab_item.1[2], slab_item.1[3]
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn needs_header(one_header: bool, term_height: u16, line_count: u64) -> bool {
    !one_header && term_height > 0 && (line_count + 3).is_multiple_of(term_height as u64)
}

#[cfg(target_os = "linux")]
fn print_slab_header() {
    println!(
        "{:<24} {:>6} {:>6} {:>6} {:>6}",
        "Cache", "Num", "Total", "Size", "Pages"
    );
}

#[cfg(target_os = "linux")]
fn print_disk_header(wide: bool) {
    if wide {
        println!("disk- -------------------reads------------------- -------------------writes------------------ ------IO-------");
        println!(
            "{:>15} {:>9} {:>11} {:>11} {:>9} {:>9} {:>11} {:>11} {:>7} {:>7}",
            "total", "merged", "sectors", "ms", "total", "merged", "sectors", "ms", "cur", "sec"
        );
    } else {
        println!("disk- ------------reads------------ ------------writes----------- -----IO------");
        println!(
            "{:>12} {:>6} {:>7} {:>7} {:>6} {:>6} {:>7} {:>7} {:>6} {:>6}",
            "total", "merged", "sectors", "ms", "total", "merged", "sectors", "ms", "cur", "sec"
        );
    }
}

#[cfg(target_os = "linux")]
fn print_disk(wide: bool, one_header: bool, term_height: u16) -> UResult<()> {
    let disk_data = DiskStat::current()
        .map_err(|_| USimpleError::new(1, "Unable to retrieve disk statistics"))?;

    let mut line_count = 0;

    print_disk_header(wide);

    for disk in disk_data {
        if !disk.is_disk() {
            continue;
        }

        if needs_header(one_header, term_height, line_count) {
            print_disk_header(wide);
        }
        line_count += 1;

        if wide {
            println!(
                "{:<5} {:>9} {:>9} {:>11} {:>11} {:>9} {:>9} {:>11} {:>11} {:>7} {:>7}",
                disk.device,
                disk.reads_completed,
                disk.reads_merged,
                disk.sectors_read,
                disk.milliseconds_spent_reading,
                disk.writes_completed,
                disk.writes_merged,
                disk.sectors_written,
                disk.milliseconds_spent_writing,
                disk.ios_currently_in_progress / 1000,
                disk.milliseconds_spent_doing_ios / 1000
            );
        } else {
            println!(
                "{:<5} {:>6} {:>6} {:>7} {:>7} {:>6} {:>6} {:>7} {:>7} {:>6} {:>6}",
                disk.device,
                disk.reads_completed,
                disk.reads_merged,
                disk.sectors_read,
                disk.milliseconds_spent_reading,
                disk.writes_completed,
                disk.writes_merged,
                disk.sectors_written,
                disk.milliseconds_spent_writing,
                disk.ios_currently_in_progress / 1000,
                disk.milliseconds_spent_doing_ios / 1000
            );
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_disk_sum() -> UResult<()> {
    let data = get_disk_sum()?;

    data.iter()
        .for_each(|(name, value)| println!("{value:>13} {name}"));

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_partition(device: &str) -> UResult<()> {
    let disk_data = DiskStat::current()
        .map_err(|_| USimpleError::new(1, "Unable to retrieve disk statistics"))?;

    let disk = disk_data
        .iter()
        .find(|disk| disk.device == device)
        .ok_or_else(|| USimpleError::new(1, format!("Disk/Partition {device} not found")))?;

    println!(
        "{device:<9} {:>11} {:>17} {:>11} {:>17}",
        "reads", "read sectors", "writes", "requested writes"
    );
    println!(
        "{:>21} {:>17} {:>11} {:>17}",
        disk.reads_completed, disk.sectors_read, disk.writes_completed, disk.sectors_written
    );

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_header(pickers: &[Picker]) {
    let mut section: Vec<&str> = vec![];
    let mut title: Vec<&str> = vec![];

    pickers.iter().for_each(|p| {
        section.push(p.0 .0.as_str());
        title.push(p.0 .1.as_str());
    });
    println!("{}", section.join(" "));
    println!("{}", title.join(" "));
}

#[cfg(target_os = "linux")]
fn print_data(
    pickers: &[Picker],
    proc_data: &ProcData,
    proc_data_before: Option<&ProcData>,
    matches: &ArgMatches,
) {
    let mut data: Vec<String> = vec![];
    let mut data_len_excess = 0;
    pickers.iter().for_each(|f| {
        f.1(
            proc_data,
            proc_data_before,
            matches,
            &mut data,
            &mut data_len_excess,
        );
    });
    println!("{}", data.join(" "));
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Report virtual memory statistics")
        .override_usage("vmstat [options]")
        .infer_long_args(true)
        .args([
            arg!(<delay> "The delay between updates in seconds")
                .required(false)
                .value_parser(value_parser!(u64).range(1..)),
            arg!(<count> "Number of updates")
                .required(false)
                .value_parser(value_parser!(u64)),
            arg!(-a --active "Display active and inactive memory"),
            arg!(-f --forks "switch displays the number of forks since boot")
                .conflicts_with_all(["slabs", "stats", "disk", "disk-sum", "partition"]),
            arg!(-m --slabs "Display slabinfo")
                .conflicts_with_all(["forks", "stats", "disk", "disk-sum", "partition"]),
            arg!(-n --"one-header" "Display the header only once rather than periodically"),
            arg!(-s --stats "Displays a table of various event counters and memory statistics")
                .conflicts_with_all(["forks", "slabs", "disk", "disk-sum", "partition"]),
            arg!(-d --disk "Report disk statistics")
                .conflicts_with_all(["forks", "slabs", "stats", "disk-sum", "partition"]),
            arg!(-D --"disk-sum" "Report some summary statistics about disk activity")
                .conflicts_with_all(["forks", "slabs", "stats", "disk", "partition"]),
            arg!(-p --partition <device> "Detailed statistics about partition")
                .conflicts_with_all(["forks", "slabs", "stats", "disk", "disk-sum"]),
            arg!(-S --unit <character> "Switches outputs between 1000 (k), 1024 (K), 1000000 (m), or 1048576 (M) bytes"),
            arg!(-t --timestamp "Append timestamp to each line"),
            arg!(-w --wide "Wide output mode"),
            arg!(-y --"no-first" "Omits first report with statistics since system boot"),
        ])
}
