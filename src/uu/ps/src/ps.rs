// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod collector;
mod picker;

#[cfg(target_os = "linux")]
use clap::crate_version;
use clap::{Arg, ArgAction, ArgMatches, Command};
use prettytable::{format::consts::FORMAT_CLEAN, Row, Table};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use uu_pgrep::process::walk_process;
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("ps.md");
const USAGE: &str = help_usage!("ps.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let snapshot = walk_process()
        .map(|it| Rc::new(RefCell::new(it)))
        .collect::<Vec<_>>();
    let mut proc_infos = Vec::new();

    proc_infos.extend(collector::basic_collector(&snapshot));
    proc_infos.extend(collector::process_collector(&matches, &snapshot));
    proc_infos.extend(collector::session_collector(&matches, &snapshot));

    // Collect codes with order
    let codes = collect_codes(&matches);

    // Collect code-header mapping
    let code_mapping = collect_head_mapping(&matches);

    // Check if the `code` exists
    for code in &codes {
        if !code_mapping.keys().any(|it| code == it) {
            return Err(USimpleError::new(
                1,
                format!("error: unknown user-defined format specifier \"{}\"", code),
            ));
        }
    }

    // Collect pickers ordered by codes
    let picker = picker::collect_picker(&codes);

    // Constructing table
    let mut rows = Vec::new();
    for proc in proc_infos {
        let picked = picker.iter().map(|f| f(proc.clone())).collect::<Vec<_>>();
        rows.push(Row::from_iter(picked));
    }

    // Apply header mapping
    let header = codes.iter().flat_map(|it| code_mapping.get(it));

    // Apply header
    let mut table = Table::from_iter([Row::from_iter(header)]);
    table.set_format(*FORMAT_CLEAN);
    table.extend(rows);

    // TODO: Sorting

    println!("{}", table);

    Ok(())
}

/// This function will extract all the needed headers from matches (the data being needed)
///
/// The headers are sequential, and the order about the final output is related to the headers
fn collect_codes(matches: &ArgMatches) -> Vec<String> {
    let mut mapping = Vec::new();

    let mut append = |code: &str| mapping.push(code.into());

    // Default header
    append("pid");
    append("tname");
    append("time");
    append("ucmd");

    mapping
}

/// Aims to collect mapping from argument
fn collect_head_mapping(matches: &ArgMatches) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    let mut append = |code: &str, header: &str| mapping.insert(code.into(), header.into());

    append("%cpu", "%CPU");
    append("%mem", "%MEM");
    append("ag_id", "AGID");
    append("ag_nice", "AGNI");
    append("args", "COMMAND");
    append("blocked", "BLOCKED");
    append("bsdstart", "START");
    append("bsdtime", "TIME");
    append("c", "C");
    append("caught", "CAUGHT");
    append("cgname", "CGNAME");
    append("cgroup", "CGROUP");
    append("cgroupns", "CGROUPNS");
    append("class", "CLS");
    append("cls", "CLS");
    append("cmd", "CMD");
    append("comm", "COMMAND");
    append("command", "COMMAND");
    append("cp", "CP");
    append("cputime", "TIME");
    append("cputimes", "TIME");
    append("cuc", "%CUC");
    append("cuu", "%CUU");
    append("drs", "DRS");
    append("egid", "EGID");
    append("egroup", "EGROUP");
    append("eip", "EIP");
    append("esp", "ESP");
    append("etime", "ELAPSED");
    append("etimes", "ELAPSED");
    append("euid", "EUID");
    append("euser", "EUSER");
    append("exe", "EXE");
    append("f", "F");
    append("fgid", "FGID");
    append("fgroup", "FGROUP");
    append("flag", "F");
    append("flags", "F");
    append("fname", "COMMAND");
    append("fuid", "FUID");
    append("fuser", "FUSER");
    append("gid", "GID");
    append("group", "GROUP");
    append("ignored", "IGNORED");
    append("ipcns", "IPCNS");
    append("label", "LABEL");
    append("lstart", "STARTED");
    append("lsession", "SESSION");
    append("luid", "LUID");
    append("lwp", "LWP");
    append("lxc", "LXC");
    append("machine", "MACHINE");
    append("maj_flt", "MAJFLT");
    append("min_flt", "MINFLT");
    append("mntns", "MNTNS");
    append("netns", "NETNS");
    append("ni", "NI");
    append("nice", "NI");
    append("nlwp", "NLWP");
    append("numa", "NUMA");
    append("nwchan", "WCHAN");
    append("oom", "OOM");
    append("oomadj", "OOMADJ");
    append("ouid", "OWNER");
    append("pcpu", "%CPU");
    append("pending", "PENDING");
    append("pgid", "PGID");
    append("pgrp", "PGRP");
    append("pid", "PID");
    append("pidns", "PIDNS");
    append("pmem", "%MEM");
    append("policy", "POL");
    append("ppid", "PPID");
    append("pri", "PRI");
    append("psr", "PSR");
    append("pss", "PSS");
    append("rbytes", "RBYTES");
    append("rchars", "RCHARS");
    append("rgid", "RGID");
    append("rgroup", "RGROUP");
    append("rops", "ROPS");
    append("rss", "RSS");
    append("rssize", "RSS");
    append("rsz", "RSZ");
    append("rtprio", "RTPRIO");
    append("ruid", "RUID");
    append("ruser", "RUSER");
    append("s", "S");
    append("sched", "SCH");
    append("seat", "SEAT");
    append("sess", "SESS");
    append("sgi_p", "P");
    append("sgid", "SGID");
    append("sgroup", "SGROUP");
    append("sid", "SID");
    append("sig", "PENDING");
    append("sigcatch", "CAUGHT");
    append("sigignore", "IGNORED");
    append("sigmask", "BLOCKED");
    append("size", "SIZE");
    append("slice", "SLICE");
    append("spid", "SPID");
    append("stackp", "STACKP");
    append("start", "STARTED");
    append("start_time", "START");
    append("stat", "STAT");
    append("state", "S");
    append("stime", "STIME");
    append("suid", "SUID");
    append("supgid", "SUPGID");
    append("supgrp", "SUPGRP");
    append("suser", "SUSER");
    append("svgid", "SVGID");
    append("svuid", "SVUID");
    append("sz", "SZ");
    append("tgid", "TGID");
    append("thcount", "THCNT");
    append("tid", "TID");
    append("time", "TIME");
    append("timens", "TIMENS");
    append("times", "TIME");
    append("tname", "TTY");
    append("tpgid", "TPGID");
    append("trs", "TRS");
    append("tt", "TT");
    append("tty", "TT");
    append("ucmd", "CMD");
    append("ucomm", "COMMAND");
    append("uid", "UID");
    append("uname", "USER");
    append("unit", "UNIT");
    append("user", "USER");
    append("userns", "USERNS");
    append("uss", "USS");
    append("utsns", "UTSNS");
    append("uunit", "UUNIT");
    append("vsize", "VSZ");
    append("vsz", "VSZ");
    append("wbytes", "WBYTES");
    append("wcbytes", "WCBYTES");
    append("wchan", "WCHAN");
    append("wchars", "WCHARS");
    append("wops", "WOPS");

    mapping
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .arg(Arg::new("help").long("help").action(ArgAction::Help))
        .args([
            Arg::new("A")
                .short('A')
                .help("all processes")
                .visible_alias("e")
                .action(ArgAction::SetTrue),
            Arg::new("a")
                .short('a')
                .help("all with tty, except session leaders")
                .action(ArgAction::SetTrue),
            // Arg::new("a_")
            //     .short('a')
            //     .help("all with tty, including other users")
            //     .action(ArgAction::SetTrue)
            //     .allow_hyphen_values(true),
            Arg::new("d")
                .short('d')
                .help("all except session leaders")
                .action(ArgAction::SetTrue),
            Arg::new("deselect")
                .long("deselect")
                .short('N')
                .help("negate selection")
                .action(ArgAction::SetTrue),
            // Arg::new("r")
            //     .short('r')
            //     .action(ArgAction::SetTrue)
            //     .help("only running processes")
            //     .allow_hyphen_values(true),
            // Arg::new("T")
            //     .short('T')
            //     .action(ArgAction::SetTrue)
            //     .help("all processes on this terminal")
            //     .allow_hyphen_values(true),
            // Arg::new("x")
            //     .short('x')
            //     .action(ArgAction::SetTrue)
            //     .help("processes without controlling ttys")
            //     .allow_hyphen_values(true),
        ])
    // .args([
    //     Arg::new("command").short('c').help("command name"),
    //     Arg::new("GID")
    //         .short('G')
    //         .long("Group")
    //         .help("real group id or name"),
    //     Arg::new("group")
    //         .short('g')
    //         .long("group")
    //         .help("session or effective group name"),
    //     Arg::new("PID").short('p').long("pid").help("process id"),
    //     Arg::new("pPID").long("ppid").help("parent process id"),
    //     Arg::new("qPID")
    //         .short('q')
    //         .long("quick-pid")
    //         .help("process id"),
    //     Arg::new("session")
    //         .short('s')
    //         .long("sid")
    //         .help("session id"),
    //     Arg::new("t").short('t').long("tty").help("terminal"),
    //     Arg::new("eUID")
    //         .short('u')
    //         .long("user")
    //         .help("effective user id or name"),
    //     Arg::new("rUID")
    //         .short('U')
    //         .long("User")
    //         .help("real user id or name"),
    // ])
}
