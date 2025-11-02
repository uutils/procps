// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::parser::OptionalKeyValue;
use std::collections::HashMap;

pub(crate) fn collect_code_mapping(formats: &[OptionalKeyValue]) -> Vec<(String, String)> {
    let mapping = default_mapping();

    formats
        .iter()
        .map(|it| {
            let key = it.key().to_string();
            match it.value() {
                Some(value) => (key, value.clone()),
                None => (key.clone(), mapping.get(&key).unwrap().clone()),
            }
        })
        .collect()
}

/// Returns the default codes.
pub(crate) fn default_codes() -> Vec<String> {
    ["pid", "tname", "time", "ucmd"].map(Into::into).to_vec()
}

/// Returns the full format codes (for -f flag).
pub(crate) fn full_format_codes() -> Vec<String> {
    [
        "uid_hack", "pid", "ppid", "c", "stime", "tname", "time", "cmd",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the extra full format codes (for -F flag).
pub(crate) fn extra_full_format_codes() -> Vec<String> {
    [
        "uid", "pid", "ppid", "c", "sz", "rss", "psr", "stime", "tname", "time", "ucmd",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the job format codes (for -j flag).
pub(crate) fn job_format_codes() -> Vec<String> {
    ["pid", "pgid", "sid", "tname", "time", "ucmd"]
        .map(Into::into)
        .to_vec()
}

/// Returns the long format codes (for -l flag).
pub(crate) fn long_format_codes() -> Vec<String> {
    [
        "f", "s", "uid", "pid", "ppid", "c", "pri", "ni", "addr", "sz", "wchan", "tname", "time",
        "ucmd",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the modified long format codes (for -ly flags).
pub(crate) fn long_y_format_codes() -> Vec<String> {
    [
        "s", "uid", "pid", "ppid", "c", "pri", "ni", "rss", "sz", "wchan", "tname", "time", "ucmd",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the default codes with PSR column (for -P flag).
pub(crate) fn default_with_psr_codes() -> Vec<String> {
    ["pid", "psr", "tname", "time", "ucmd"]
        .map(Into::into)
        .to_vec()
}

/// Returns the signal format codes (for -s flag).
pub(crate) fn signal_format_codes() -> Vec<String> {
    [
        "uid", "pid", "pending", "blocked", "ignored", "caught", "stat", "tname", "time", "command",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the user format codes (for -u flag).
pub(crate) fn user_format_codes() -> Vec<String> {
    [
        "user", "pid", "%cpu", "%mem", "vsz", "rss", "tname", "stat", "bsdstart", "time", "command",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the virtual memory format codes (for -v flag).
pub(crate) fn vm_format_codes() -> Vec<String> {
    [
        "pid", "tname", "stat", "time", "maj_flt", "trs", "drs", "rss", "%mem", "command",
    ]
    .map(Into::into)
    .to_vec()
}

/// Returns the register format codes (for -X flag).
pub(crate) fn register_format_codes() -> Vec<String> {
    [
        "pid", "stackp", "esp", "eip", "tmout", "alarm", "stat", "tname", "time", "command",
    ]
    .map(Into::into)
    .to_vec()
}

/// Collect mapping from argument
pub(crate) fn default_mapping() -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    let mut append = |code: &str, header: &str| mapping.insert(code.into(), header.into());

    // This list is mainly generated from both `ps L` output and manpage,
    // but some are also apparently undocumented.
    append("%cpu", "%CPU");
    append("%mem", "%MEM");
    append("_left", "LLLLLLLL");
    append("_left2", "L2L2L2L2");
    append("_right", "RRRRRRRR");
    append("_right2", "R2R2R2R2");
    append("_unlimited", "U");
    append("_unlimited2", "U2");
    append("addr", "ADDR"); // undocumented
    append("ag_id", "AGID");
    append("ag_nice", "AGNI");
    append("alarm", "ALARM"); // undocumented
    append("args", "COMMAND");
    append("atime", "TIME");
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
    append("context", "CONTEXT");
    append("cp", "CP");
    append("cpuid", "CPUID");
    append("cputime", "TIME");
    append("cputimes", "TIME");
    append("cuc", "%CUC");
    append("cuu", "%CUU");
    append("docker", "DOCKER");
    append("drs", "DRS");
    append("dsiz", "DSIZ");
    append("egid", "EGID");
    append("egroup", "EGROUP");
    append("eip", "EIP");
    append("environ", "ENVIRONM");
    append("esp", "ESP");
    append("etime", "ELAPSED");
    append("etimes", "ELAPSED");
    append("euid", "EUID");
    append("euser", "EUSER");
    append("exe", "EXE");
    append("f", "F");
    append("fds", "FDS");
    append("fgid", "FGID");
    append("fgroup", "FGROUP");
    append("flag", "F");
    append("flags", "F");
    append("fname", "COMMAND");
    append("fsgid", "FSGID");
    append("fsgroup", "FSGROUP");
    append("fsuid", "FSUID");
    append("fsuser", "FSUSER");
    append("fuid", "FUID");
    append("fuser", "FUSER");
    append("gid", "GID");
    append("group", "GROUP");
    append("htprv", "HTPRV");
    append("htshr", "HTSHR");
    append("ignored", "IGNORED");
    append("intpri", "PRI");
    append("ipcns", "IPCNS");
    append("label", "LABEL");
    append("lastcpu", "C");
    append("lim", "LIM");
    append("longtname", "TTY");
    append("lsession", "SESSION");
    append("lstart", "STARTED");
    append("luid", "LUID");
    append("lwp", "LWP");
    append("lxc", "LXC");
    append("m_drs", "DRS");
    append("m_size", "SIZE");
    append("m_trs", "TRS");
    append("machine", "MACHINE");
    append("maj_flt", "MAJFL");
    append("majflt", "MAJFLT");
    append("min_flt", "MINFL");
    append("minflt", "MINFLT");
    append("mntns", "MNTNS");
    append("netns", "NETNS");
    append("ni", "NI");
    append("nice", "NI");
    append("nlwp", "NLWP");
    append("numa", "NUMA");
    append("nwchan", "WCHAN");
    append("oom", "OOM");
    append("oomadj", "OOMADJ");
    append("opri", "PRI");
    append("ouid", "OWNER");
    append("pagein", "PAGEIN");
    append("pcap", "PCAP");
    append("pcaps", "PCAPS");
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
    append("pri_api", "API");
    append("pri_bar", "BAR");
    append("pri_baz", "BAZ");
    append("pri_foo", "FOO");
    append("priority", "PRI");
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
    append("session", "SESS");
    append("sgi_p", "P");
    append("sgi_rss", "RSS");
    append("sgid", "SGID");
    append("sgroup", "SGROUP");
    append("sid", "SID");
    append("sig", "PENDING");
    append("sig_block", "BLOCKED");
    append("sig_catch", "CATCHED");
    append("sig_ignore", "IGNORED");
    append("sig_pend", "SIGNAL");
    append("sigcatch", "CAUGHT");
    append("sigignore", "IGNORED");
    append("sigmask", "BLOCKED");
    append("size", "SIZE");
    append("slice", "SLICE");
    append("spid", "SPID");
    append("stackp", "STACKP");
    append("start", "STARTED");
    append("start_stack", "STACKP");
    append("start_time", "START");
    append("stat", "STAT");
    append("state", "S");
    append("stime", "STIME");
    append("suid", "SUID");
    append("supgid", "SUPGID");
    append("supgrp", "SUPGRP");
    append("suser", "SUSER");
    append("svgid", "SVGID");
    append("svgroup", "SVGROUP");
    append("svuid", "SVUID");
    append("svuser", "SVUSER");
    append("sz", "SZ");
    append("tgid", "TGID");
    append("thcount", "THCNT");
    append("tid", "TID");
    append("time", "TIME");
    append("timens", "TIMENS");
    append("times", "TIME");
    append("tmout", "TMOUT"); // undocumented
    append("tname", "TTY");
    append("tpgid", "TPGID");
    append("trs", "TRS");
    append("trss", "TRSS");
    append("tsig", "PENDING");
    append("tsiz", "TSIZ");
    append("tt", "TT");
    append("tty", "TT");
    append("tty4", "TTY");
    append("tty8", "TTY");
    append("ucmd", "CMD");
    append("ucomm", "COMMAND");
    append("uid", "UID");
    append("uid_hack", "UID");
    append("uname", "USER");
    append("unit", "UNIT");
    append("user", "USER");
    append("userns", "USERNS");
    append("uss", "USS");
    append("util", "C");
    append("utsns", "UTSNS");
    append("uunit", "UUNIT");
    append("vsize", "VSZ");
    append("vsz", "VSZ");
    append("wbytes", "WBYTES");
    append("wcbytes", "WCBYTES");
    append("wchan", "WCHAN");
    append("wchars", "WCHARS");
    append("wname", "WCHAN");
    append("wops", "WOPS");
    append("zone", "ZONE");

    mapping
}
