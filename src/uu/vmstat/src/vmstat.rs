// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod parser;
mod picker;

#[cfg(target_os = "linux")]
use crate::picker::{get_pickers, Picker};
use clap::{arg, crate_version, ArgMatches, Command};
#[allow(unused_imports)]
pub use parser::*;
#[allow(unused_imports)]
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("vmstat.md");
const USAGE: &str = help_usage!("vmstat.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    #[allow(unused)]
    let matches = uu_app().try_get_matches_from(args)?;
    #[cfg(target_os = "linux")]
    {
        let one_header = matches.get_flag("one-header");
        let no_first = matches.get_flag("no-first");
        let delay = matches.get_one::<String>("delay");
        let mut delay = if let Some(delay) = delay {
            let delay = delay.parse::<i64>().unwrap();
            if delay <= 0 {
                return Err(USimpleError::new(-1, "delay must be greater than 0"));
            }
            delay
        } else {
            -1
        };
        let count = matches.get_one::<String>("count");
        let mut count = if let Some(count) = count {
            let mut count = count.parse::<i64>().unwrap();
            if count < 0 {
                return Err(USimpleError::new(-1, "count must be greater than 0"));
            }
            if count == 0 {
                count = 1;
            }
            count
        } else {
            -1
        };
        if count < 0 && delay < 0 {
            count = 1;
            if no_first {
                delay = 1;
                count = 1;
            }
        }

        let pickers = get_pickers(&matches);
        let mut proc_data = ProcData::new();

        let mut line_count = 0;
        print_header(&pickers);
        if !no_first {
            print_data(&pickers, &proc_data, None, &matches);
            line_count += 1;
        }

        let term_height = terminal_size::terminal_size()
            .map(|size| size.1 .0)
            .unwrap_or(0);

        while count < 0 || line_count < count {
            std::thread::sleep(std::time::Duration::from_secs(delay as u64));
            let proc_data_now = ProcData::new();
            if !one_header && term_height > 0 && ((line_count + 3) % term_height as i64 == 0) {
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
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            arg!(<delay> "The delay between updates in seconds").required(false),
            arg!(<count> "Number of updates").required(false),
            arg!(-a --active "Display active and inactive memory"),
            // arg!(-f --forks "switch displays the number of forks since boot"),
            // arg!(-m --slabs "Display slabinfo"),
            arg!(-n --"one-header" "Display the header only once rather than periodically"),
            // arg!(-s --stats "Displays a table of various event counters and memory statistics"),
            // arg!(-d --disk "Report disk statistics"),
            // arg!(-D --"disk-sum" "Report some summary statistics about disk activity"),
            // arg!(-p --partition <device> "Detailed statistics about partition"),
            arg!(-S --unit <character> "Switches outputs between 1000 (k), 1024 (K), 1000000 (m), or 1048576 (M) bytes"),
            // arg!(-t --timestamp "Append timestamp to each line"),
            arg!(-w --wide "Wide output mode"),
            arg!(-y --"no-first" "Omits first report with statistics since system boot"),
        ])
}
