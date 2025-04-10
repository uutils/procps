// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, ArgAction, Command};
use std::env;
use std::fs;
use std::io::Error;
use std::path::Path;
use std::process;
use uu_pmap::smaps_format_parser::parse_smap_entries;
use uu_pmap::smaps_format_parser::SmapEntry;
use uu_top::header;
use uucore::uptime::get_formatted_time;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("hugetop.md");
const USAGE: &str = help_usage!("hugetop.md");

#[derive(Debug)]
struct ProcessHugepageInfo {
    pid: u32,
    name: String,
    entries: Vec<SmapEntry>,
}

#[derive(Default, Debug)]
struct HugePageSizeInfo {
    size_kb: u64,
    free: u64,
    total: u64,
}

impl std::fmt::Display for HugePageSizeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_str = match self.size_kb {
            2048 => "2Mi",
            1048576 => "1Gi",
            _ => panic!("{}", self.size_kb),
        };

        write!(f, "{} - {}/{}", size_str, self.free, self.total)
    }
}

fn parse_hugepage() -> Result<Vec<HugePageSizeInfo>, Error> {
    let parse_hugepage_value = |p: &Path| -> Result<u64, Error> {
        fs::read_to_string(p)?.trim().parse().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid memory info format",
            )
        })
    };

    let info_dir = fs::read_dir("/sys/kernel/mm/hugepages")?;

    let mut sizes = Vec::new();

    for entry in info_dir {
        let entry = entry?;

        let mut info = HugePageSizeInfo::default();

        info.total = parse_hugepage_value(&entry.path().join("nr_hugepages"))?;
        info.free = parse_hugepage_value(&entry.path().join("free_hugepages"))?;
        info.size_kb = entry
            .file_name()
            .into_string()
            .unwrap()
            .split("-")
            .nth(1)
            .unwrap()
            .replace("kB", "")
            .parse()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid memory info format",
                )
            })?;

        sizes.push(info);
    }

    Ok(sizes)
}

fn parse_process_info(p: &fs::DirEntry) -> Option<ProcessHugepageInfo> {
    let pid_str = p.file_name().into_string().unwrap_or_default();

    // Skip non-PID directories
    let pid = pid_str.parse::<u32>().ok()?;

    // Parse name
    let name = fs::read_to_string(p.path().join("status"))
        .ok()?
        .lines()
        .nth(0)
        .unwrap_or_default()
        .split(":")
        .nth(1)
        .unwrap_or_default()
        .trim()
        .to_string();

    let contents = fs::read_to_string(p.path().join("smaps")).ok()?;
    let smap_entries = parse_smap_entries(&contents).ok()?;
    let smap_entries: Vec<_> = smap_entries
        .into_iter()
        .filter(|entry| entry.kernel_page_size_in_kb >= 2024)
        .collect();

    if smap_entries.is_empty() {
        return None;
    }

    Some(ProcessHugepageInfo {
        name,
        pid,
        entries: smap_entries,
    })
}

#[cfg(target_os = "linux")]
fn parse_process_hugepages() -> Result<Vec<ProcessHugepageInfo>, Error> {
    let mut processes = Vec::new();
    let proc_dir = fs::read_dir("/proc")?;

    for entry in proc_dir {
        let entry = entry?;
        if let Some(info) = parse_process_info(&entry) {
            processes.push(info);
        }
    }

    Ok(processes)
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    match parse_hugepage() {
        Ok(sys_info) => match parse_process_hugepages() {
            Ok(p_info) => {
                print!("{}", construct_str(sys_info, &p_info,));
            }
            Err(e) => {
                eprintln!("hugetop: failed to read process hugepage info: {}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("hugetop: failed to read hugepage info: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .args_override_self(true)
        .infer_long_args(true)
        .disable_help_flag(true)
        .arg(arg!(--help "display this help and exit").action(ArgAction::SetTrue))
}

fn construct_str(sys: Vec<HugePageSizeInfo>, processes: &[ProcessHugepageInfo]) -> String {
    let mut output = String::new();

    output.push_str(&construct_system_str(sys));
    output.push_str(&format_process_str(processes));

    output
}

fn format_process_str(processes: &[ProcessHugepageInfo]) -> String {
    let mut output = String::new();
    let header = format!(
        "{:<8} {:<12} {:<12} {:<12}\n",
        "PID", "Private", "Shared", "Process"
    );

    output.push_str(&header);

    for process in processes {
        for smap_entry in &process.entries {
            output.push_str(&format!(
                "{:<8} {:<12} {:<12} {:<12}\n",
                process.pid,
                smap_entry.private_hugetlb_in_kb,
                smap_entry.shared_hugetlb_in_kb,
                process.name
            ));
        }
    }

    output
}

fn construct_system_str(sys: Vec<HugePageSizeInfo>) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "top - {time} {uptime}, {user}\n",
        time = get_formatted_time(),
        uptime = header::uptime(),
        user = header::user(),
    ));

    for (i, info) in sys.iter().enumerate() {
        if i < sys.len() - 1 {
            output.push_str(&format!("{}, ", info.to_string()));
        } else {
            output.push_str(&info.to_string());
            output.push('\n');
        }
    }

    output
}
