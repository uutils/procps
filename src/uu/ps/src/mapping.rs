// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::collections::HashMap;
use crate::parser::OptionalKeyValue;

pub(crate) fn apply_format_mapping(formats: &[OptionalKeyValue]) -> HashMap<String, String> {
    let mut mapping = default_mapping();

    for optional_kv in formats {
        let key = optional_kv.key();
        if !optional_kv.is_value_empty() {
            mapping.insert(key.to_owned(), optional_kv.try_get::<String>().unwrap());
        }
    }

    mapping
}

/// This function will extract all the needed headers from matches (the data being needed)
///
/// The headers are sequential, and the order about the final output is related to the headers
pub(crate) fn default_codes() -> Vec<String> {
    let mut mapping = Vec::new();
    let mut append = |code: &str| mapping.push(code.into());

    append("pid");
    append("tname");
    append("time");
    append("ucmd");

    mapping
}

/// Collect mapping from argument
///
/// TODO: collecting mapping from matches
pub(crate) fn default_mapping() -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    let mut append = |code: &str, header: &str| mapping.insert(code.into(), header.into());

    // Those mapping generated from manpage
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
