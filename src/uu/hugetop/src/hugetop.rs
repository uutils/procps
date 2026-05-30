// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use chrono::{DateTime, Local};
use clap::{value_parser, Arg, Command};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use uucore::error::UResult;

const DEFAULT_HUGEPAGES_ROOT: &str = "/sys/kernel/mm/hugepages";
const SYS_NODES_ROOT: &str = "/sys/devices/system/node";
const DEFAULT_PROC_ROOT: &str = "/proc";

/// Hugepage statistics from /proc/[pid]/smaps_rollup file.
///
/// These values represent:
/// - `0`: AnonHugePages (Anonymous Hugepage memory in kB)
/// - `1`: Shared_Hugetlb (Hugetlb memory shared with other processes in kB)
/// - `2`: Private_Hugetlb (Hugetlb memory private to the process in kB)
///
/// See Linux kernel documentation:
/// - https://www.kernel.org/doc/html/latest/filesystems/proc.html#proc-pid-smaps-smaps-rollup
type SmapsRollupValues = (u64, u64, u64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct HugePagePool {
    size_kb: u64,
    total_pages: u64,
    free_pages: u64,
    reserved_pages: u64,
    surplus_pages: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessHugeUsage {
    pid: u32,
    command: String,
    anon_huge_kb: u64,
    shared_hugetlb_kb: u64,
    private_hugetlb_kb: u64,
}

impl ProcessHugeUsage {
    fn total_kb(&self) -> u64 {
        self.anon_huge_kb + self.shared_hugetlb_kb + self.private_hugetlb_kb
    }
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let limit = matches.get_one::<usize>("lines").copied();
    let numa = matches.get_flag("numa");
    let human = matches.get_flag("human");
    let once = matches.get_flag("once");
    let delay = *matches.get_one::<u64>("delay").unwrap_or(&0);

    if once || delay == 0 {
        run(numa, human, limit)
    } else {
        loop {
            // Clear the terminal to roughly approximate hugetop's screen refresh behavior.
            print!("\x1B[2J\x1B[H");
            run(numa, human, limit)?;
            sleep(Duration::from_secs(delay));
        }
    }
}

fn run(numa: bool, human: bool, limit: Option<usize>) -> UResult<()> {
    print_summary(numa, human);
    print_headings();
    print_procs(human, limit);
    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .about("Report hugepage usage of processes and the system as a whole")
        .arg(
            Arg::new("delay")
                .short('d')
                .long("delay")
                .value_name("SECONDS")
                .help("Delay between updates (0 = run once)")
                .value_parser(value_parser!(u64)),
        )
        .arg(
            Arg::new("numa")
                .short('n')
                .long("numa")
                .help("Display per NUMA node huge page information")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("once")
                .short('o')
                .long("once")
                .help("Only display once, then exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("human")
                .short('H')
                .long("human")
                .help("Display human-readable output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lines")
                .short('l')
                .long("lines")
                .value_name("N")
                .help("Show the top N processes")
                .value_parser(value_parser!(usize)),
        )
}

fn print_summary(numa: bool, human: bool) {
    let now: DateTime<Local> = Local::now();
    println!("hugetop - {}", now.format("%a %b %e %T %Y"));

    let pools = match read_node_hugepage_pools() {
        Ok(nodes) if numa => {
            for (node, pools) in &nodes {
                print_node(node, pools, human);
            }
            return;
        }
        Ok(nodes) => merge_node_pools(&nodes),
        Err(_) => Vec::new(),
    };

    if pools.is_empty() {
        let pools = read_hugepage_pools(Path::new(DEFAULT_HUGEPAGES_ROOT)).unwrap_or_default();
        if pools.is_empty() {
            println!("(no hugepage pools found)");
            return;
        }
        print_node("node(s)", &pools, human);
    } else {
        print_node("node(s)", &pools, human);
    }
}

fn print_headings() {
    println!("{:>8} {:>10} {:>10} COMMAND", "PID", "SHARED", "PRIVATE");
}

fn print_procs(human: bool, limit: Option<usize>) {
    let mut processes =
        read_process_hugepage_usage(Path::new(DEFAULT_PROC_ROOT)).unwrap_or_default();

    processes.sort_by_key(|usage| std::cmp::Reverse(usage.total_kb()));

    let limit = limit.unwrap_or(processes.len());

    for (shown, usage) in processes.into_iter().enumerate() {
        if shown >= limit {
            break;
        }

        let shared = format_kb(usage.shared_hugetlb_kb, human);
        let private = format_kb(usage.private_hugetlb_kb, human);

        println!(
            "{:>8} {:>10} {:>10} {}",
            usage.pid, shared, private, usage.command
        );
    }
}

fn print_node(node: &str, pools: &[HugePagePool], human: bool) {
    let mut line = String::new();
    line.push_str(node);
    line.push(':');

    for (i, pool) in pools.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }

        let size = if human {
            humanized(pool.size_kb, false)
        } else {
            format!("{}kB", pool.size_kb)
        };

        line.push_str(&format!(
            " {} - {}/{}",
            size, pool.free_pages, pool.total_pages
        ));
    }

    println!("{}", line);
}

fn format_kb(kb: u64, human: bool) -> String {
    if human {
        humanized(kb, false)
    } else {
        format!("{}", kb)
    }
}

fn humanized(kib: u64, si: bool) -> String {
    let b = kib * 1024;
    let units = ['B', 'K', 'M', 'G', 'T', 'P'];
    let mut level = 0;
    let mut divisor = 1u64;

    while level < units.len() - 1 && divisor * 100 <= b {
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

fn read_node_hugepage_pools() -> UResult<Vec<(String, Vec<HugePagePool>)>> {
    let mut nodes = Vec::new();
    let Ok(entries) = fs::read_dir(SYS_NODES_ROOT) else {
        return Ok(nodes);
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(n) if n.starts_with("node") => n.to_string(),
            _ => continue,
        };

        let path = entry.path().join("hugepages");
        if !path.is_dir() {
            continue;
        }

        let pools = read_hugepage_pools(&path)?;
        if pools.is_empty() {
            continue;
        }

        nodes.push((name, pools));
    }

    Ok(nodes)
}

fn merge_node_pools(nodes: &[(String, Vec<HugePagePool>)]) -> Vec<HugePagePool> {
    let mut map: BTreeMap<u64, HugePagePool> = BTreeMap::new();

    for (_, pools) in nodes {
        for pool in pools {
            let entry = map.entry(pool.size_kb).or_insert_with(|| HugePagePool {
                size_kb: pool.size_kb,
                total_pages: 0,
                free_pages: 0,
                reserved_pages: 0,
                surplus_pages: 0,
            });
            entry.total_pages += pool.total_pages;
            entry.free_pages += pool.free_pages;
            entry.reserved_pages += pool.reserved_pages;
            entry.surplus_pages += pool.surplus_pages;
        }
    }

    map.into_values().collect()
}

fn read_hugepage_pools(root: impl AsRef<Path>) -> UResult<Vec<HugePagePool>> {
    let mut pools = Vec::new();

    let Ok(entries) = fs::read_dir(&root) else {
        return Ok(pools);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(size_kb) = parse_hugepage_dir_name(name) else {
            continue;
        };

        let total_pages = read_u64(path.join("nr_hugepages"));
        let free_pages = read_u64(path.join("free_hugepages"));
        let reserved_pages = read_u64(path.join("resv_hugepages"));
        let surplus_pages = read_u64(path.join("surplus_hugepages"));

        pools.push(HugePagePool {
            size_kb,
            total_pages,
            free_pages,
            reserved_pages,
            surplus_pages,
        });
    }

    pools.sort_by_key(|pool| pool.size_kb);
    Ok(pools)
}

fn read_process_hugepage_usage(root: impl AsRef<Path>) -> UResult<Vec<ProcessHugeUsage>> {
    let mut usages = Vec::new();

    let Ok(entries) = fs::read_dir(&root) else {
        return Ok(usages);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let Ok(pid) = file_name.parse::<u32>() else {
            continue;
        };

        let Some((_, shared_hugetlb_kb, private_hugetlb_kb)) =
            parse_smaps_rollup(path.join("smaps_rollup"))
        else {
            continue;
        };

        let total_kb = shared_hugetlb_kb + private_hugetlb_kb;
        if total_kb == 0 {
            continue;
        }

        let command = fs::read_to_string(path.join("comm"))
            .unwrap_or_else(|_| String::from("?"))
            .trim()
            .to_string();

        usages.push(ProcessHugeUsage {
            pid,
            command,
            anon_huge_kb: 0,
            shared_hugetlb_kb,
            private_hugetlb_kb,
        });
    }

    Ok(usages)
}

fn parse_hugepage_dir_name(name: &str) -> Option<u64> {
    let prefix = "hugepages-";
    let suffix = "kB";

    if !name.starts_with(prefix) || !name.ends_with(suffix) {
        return None;
    }

    name[prefix.len()..name.len() - suffix.len()]
        .parse::<u64>()
        .ok()
}

fn parse_smaps_rollup(path: impl AsRef<Path>) -> Option<SmapsRollupValues> {
    let content = fs::read_to_string(&path).ok()?;

    let mut anon_huge_kb = 0;
    let mut shared_hugetlb_kb = 0;
    let mut private_hugetlb_kb = 0;

    for line in content.lines() {
        if let Some(value) = parse_kb_field(line, "AnonHugePages:") {
            anon_huge_kb = value;
        } else if let Some(value) = parse_kb_field(line, "Shared_Hugetlb:") {
            shared_hugetlb_kb = value;
        } else if let Some(value) = parse_kb_field(line, "Private_Hugetlb:") {
            private_hugetlb_kb = value;
        }
    }

    Some((anon_huge_kb, shared_hugetlb_kb, private_hugetlb_kb))
}

fn parse_kb_field(line: &str, field: &str) -> Option<u64> {
    let value = line.strip_prefix(field)?.trim();
    let number = value.split_whitespace().next()?;
    number.parse::<u64>().ok()
}

fn read_u64(path: impl AsRef<Path>) -> u64 {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    #[cfg(target_os = "linux")]
    fn parse_hugepage_name_works() {
        assert_eq!(parse_hugepage_dir_name("hugepages-2048kB"), Some(2048));
        assert_eq!(
            parse_hugepage_dir_name("hugepages-1048576kB"),
            Some(1_048_576)
        );
        assert_eq!(parse_hugepage_dir_name("hugepages-foo"), None);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn parse_smaps_rollup_works() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("smaps_rollup");
        let mut f = fs::File::create(&file).unwrap();
        writeln!(f, "AnonHugePages:      512 kB").unwrap();
        writeln!(f, "Shared_Hugetlb:      64 kB").unwrap();
        writeln!(f, "Private_Hugetlb:     32 kB").unwrap();

        assert_eq!(parse_smaps_rollup(&file), Some((512, 64, 32)));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn reads_pools_from_tree() {
        let dir = tempfile::tempdir().unwrap();
        let pool = dir.path().join("hugepages-2048kB");
        fs::create_dir(&pool).unwrap();
        fs::write(pool.join("nr_hugepages"), "10\n").unwrap();
        fs::write(pool.join("free_hugepages"), "3\n").unwrap();
        fs::write(pool.join("resv_hugepages"), "2\n").unwrap();
        fs::write(pool.join("surplus_hugepages"), "1\n").unwrap();

        let pools = read_hugepage_pools(dir.path()).unwrap();
        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].size_kb, 2048);
        assert_eq!(pools[0].total_pages, 10);
        assert_eq!(pools[0].free_pages, 3);
    }
}
