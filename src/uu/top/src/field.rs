// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

// This static field will used in future
#[allow(unused)]
static FIELDS: OnceLock<HashMap<String, String>> = OnceLock::new();

// Generated from manpage
#[allow(unused)]
pub(crate) fn fields() -> HashSet<String> {
    FIELDS
        .get_or_init(|| {
            vec![
                ("%CPU", "CPU Usage"),
                ("%CUC", "CPU Utilization"),
                ("%CUU", "CPU Utilization"),
                ("%MEM", "Memory Usage (RES)"),
                ("AGID", "Autogroup Identifier"),
                ("AGNI", "Autogroup Nice Value"),
                ("CGNAME", "Control Group Name"),
                ("CGROUPS", "Control Groups"),
                ("CODE", "Code Size (KiB)"),
                ("COMMAND", "Command Name or Command Line"),
                ("DATA", "Data + Stack Size (KiB)"),
                ("ELAPSED", "Elapsed Running Time"),
                ("ENVIRON", "Environment variables"),
                ("EXE", "Executable Path"),
                ("Flags", "Task Flags"),
                ("GID", "Group Id"),
                ("GROUP", "Group Name"),
                ("LOGID", "Login User Id"),
                ("LXC", "Lxc Container Name"),
                ("NI", "Nice Value"),
                ("NU", "Last known NUMA node"),
                ("OOMa", "Out of Memory Adjustment Factor"),
                ("OOMs", "Out of Memory Score"),
                ("P", "Last used CPU (SMP)"),
                ("PGRP", "Process Group Id"),
                ("PID", "Process Id"),
                ("PPID", "Parent Process Id"),
                ("PR", "Priority"),
                ("PSS", "Proportional Resident Memory, smaps (KiB)"),
            ]
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<HashMap<String, String>>()
        })
        .keys()
        .cloned()
        .collect()
}

#[allow(unused)]
pub(crate) fn description_of<T>(field: T) -> Option<String>
where
    T: Into<String>,
{
    let field: String = field.into();
    fields().get(&field).cloned()
}
