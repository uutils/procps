// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Common process matcher logic shared by pgrep, pkill and pidwait

use std::fs;
use std::hash::Hash;
use std::{collections::HashSet, io};

use clap::{arg, Arg, ArgAction, ArgMatches};
use regex::Regex;
#[cfg(unix)]
use uucore::libc::{getpgrp, getsid};
#[cfg(unix)]
use uucore::{
    display::Quotable,
    entries::{grp2gid, usr2uid},
    signals::signal_by_name_or_value,
};

use uucore::error::{UResult, USimpleError};

use crate::process::{walk_process, walk_threads, ProcessInformation, Teletype};

pub struct Settings {
    pub regex: Regex,

    pub exact: bool,
    pub full: bool,
    pub ignore_case: bool,
    pub inverse: bool,
    pub newest: bool,
    pub oldest: bool,
    pub older: Option<u64>,
    pub parent: Option<HashSet<u64>>,
    pub runstates: Option<String>,
    pub terminal: Option<HashSet<Teletype>>,
    #[cfg(unix)]
    pub signal: usize,
    pub require_handler: bool,
    pub uid: Option<HashSet<u32>>,
    pub euid: Option<HashSet<u32>>,
    pub gid: Option<HashSet<u32>>,
    pub pgroup: Option<HashSet<u64>>,
    pub session: Option<HashSet<u64>>,
    pub cgroup: Option<HashSet<String>>,
    pub threads: bool,

    pub pidfile: Option<String>,
}

pub fn get_match_settings(matches: &ArgMatches) -> UResult<Settings> {
    let pattern = try_get_pattern_from(matches)?;
    let regex = Regex::new(&pattern).map_err(|e| USimpleError::new(2, e.to_string()))?;

    let settings = Settings {
        regex,
        exact: matches.get_flag("exact"),
        full: matches.get_flag("full"),
        ignore_case: matches.get_flag("ignore-case"),
        inverse: matches.get_flag("inverse"),
        newest: matches.get_flag("newest"),
        oldest: matches.get_flag("oldest"),
        parent: matches
            .get_many::<u64>("parent")
            .map(|parents| parents.copied().collect()),
        runstates: matches.get_one::<String>("runstates").cloned(),
        older: matches.get_one::<u64>("older").copied(),
        terminal: matches.get_many::<String>("terminal").map(|ttys| {
            ttys.cloned()
                .flat_map(Teletype::try_from)
                .collect::<HashSet<_>>()
        }),
        #[cfg(unix)]
        signal: parse_signal_value(matches.get_one::<String>("signal").unwrap())?,
        require_handler: matches.get_flag("require-handler"),
        uid: matches
            .get_many::<u32>("uid")
            .map(|ids| ids.cloned().collect()),
        euid: matches
            .get_many::<u32>("euid")
            .map(|ids| ids.cloned().collect()),
        gid: matches
            .get_many::<u32>("group")
            .map(|ids| ids.cloned().collect()),
        pgroup: matches.get_many::<u64>("pgroup").map(|xs| {
            xs.map(|pg| {
                if *pg == 0 {
                    unsafe { getpgrp() as u64 }
                } else {
                    *pg
                }
            })
            .collect()
        }),
        session: matches.get_many::<u64>("session").map(|xs| {
            xs.map(|sid| {
                if *sid == 0 {
                    unsafe { getsid(0) as u64 }
                } else {
                    *sid
                }
            })
            .collect()
        }),
        cgroup: matches
            .get_many::<String>("cgroup")
            .map(|groups| groups.cloned().collect()),
        threads: false,
        pidfile: matches.get_one::<String>("pidfile").cloned(),
    };

    if !settings.newest
        && !settings.oldest
        && settings.runstates.is_none()
        && settings.older.is_none()
        && settings.parent.is_none()
        && settings.terminal.is_none()
        && settings.uid.is_none()
        && settings.euid.is_none()
        && settings.gid.is_none()
        && settings.pgroup.is_none()
        && settings.session.is_none()
        && settings.cgroup.is_none()
        && !settings.require_handler
        && settings.pidfile.is_none()
        && pattern.is_empty()
    {
        return Err(USimpleError::new(
            2,
            format!(
                "no matching criteria specified\n\
                 Try `{} --help' for more information.",
                uucore::util_name()
            ),
        ));
    }

    if !settings.full && pattern.len() > 15 {
        let msg = format!("pattern that searches for process name longer than 15 characters will result in zero matches\n\
                           Try `{} -f' option to match against the complete command line.", uucore::util_name());
        return Err(USimpleError::new(1, msg));
    }

    Ok(settings)
}

pub fn find_matching_pids(settings: &Settings) -> UResult<Vec<ProcessInformation>> {
    let mut pids = collect_matched_pids(settings)?;
    if pids.is_empty() {
        uucore::error::set_exit_code(1);
        Ok(pids)
    } else {
        Ok(process_flag_o_n(settings, &mut pids))
    }
}

/// Try to get the pattern from the command line arguments. Returns an empty string if no pattern
/// is specified.
fn try_get_pattern_from(matches: &ArgMatches) -> UResult<String> {
    let pattern = match matches.get_many::<String>("pattern") {
        Some(patterns) if patterns.len() > 1 => {
            return Err(USimpleError::new(
                2,
                format!(
                    "only one pattern can be provided\nTry `{} --help' for more information.",
                    uucore::util_name()
                ),
            ))
        }
        Some(mut patterns) => patterns.next().unwrap(),
        None => return Ok(String::new()),
    };

    let pattern = if matches.get_flag("ignore-case") {
        &pattern.to_lowercase()
    } else {
        pattern
    };

    let pattern = if matches.get_flag("exact") {
        &format!("^{}$", pattern)
    } else {
        pattern
    };

    Ok(pattern.to_string())
}

fn any_matches<T: Eq + Hash>(optional_ids: &Option<HashSet<T>>, id: T) -> bool {
    optional_ids.as_ref().is_none_or(|ids| ids.contains(&id))
}

/// Collect pids with filter construct from command line arguments
fn collect_matched_pids(settings: &Settings) -> UResult<Vec<ProcessInformation>> {
    // Filtration general parameters
    let filtered: Vec<ProcessInformation> = {
        let mut tmp_vec = Vec::new();

        let pids = if settings.threads {
            walk_threads().collect::<Vec<_>>()
        } else {
            walk_process().collect::<Vec<_>>()
        };
        let our_pid = std::process::id() as usize;
        let pid_from_pidfile = settings
            .pidfile
            .as_ref()
            .map(|filename| read_pidfile(filename))
            .transpose()?;

        for mut pid in pids {
            if pid.pid == our_pid {
                continue;
            }

            let run_state_matched = match (&settings.runstates, pid.run_state()) {
                (Some(arg_run_states), Ok(pid_state)) => {
                    arg_run_states.contains(&pid_state.to_string())
                }
                (_, Err(_)) => false,
                _ => true,
            };

            let name = pid.name().unwrap();
            let name = if settings.ignore_case {
                name.to_lowercase()
            } else {
                name
            };
            let pattern_matched = {
                let want = if settings.full {
                    // Equals `cmdline` in /proc/<pid>/cmdline
                    &pid.cmdline
                } else {
                    // Equals `Name` in /proc/<pid>/status
                    &name
                };

                settings.regex.is_match(want)
            };

            let tty_matched = any_matches(&settings.terminal, pid.tty());

            let arg_older = settings.older.unwrap_or(0);
            let older_matched = pid.start_time().unwrap() >= arg_older;

            let parent_matched = any_matches(&settings.parent, pid.ppid().unwrap());
            let pgroup_matched = any_matches(&settings.pgroup, pid.pgid().unwrap());
            let session_matched = any_matches(&settings.session, pid.sid().unwrap());
            let cgroup_matched = any_matches(
                &settings.cgroup,
                pid.cgroup_v2_path().unwrap_or("/".to_string()),
            );

            let ids_matched = any_matches(&settings.uid, pid.uid().unwrap())
                && any_matches(&settings.euid, pid.euid().unwrap())
                && any_matches(&settings.gid, pid.gid().unwrap());

            #[cfg(unix)]
            let handler_matched = if settings.require_handler {
                // Bits in SigCgt are off by one (ie. bit 0 represents signal 1, etc.)
                let mask_to_test = if settings.signal == 0 {
                    // In original pgrep, testing for signal 0 seems to return results for signal 64 instead.
                    1 << (64 - 1)
                } else {
                    1 << (settings.signal - 1)
                };
                let mask =
                    u64::from_str_radix(pid.clone().status().get("SigCgt").unwrap(), 16).unwrap();
                mask & mask_to_test != 0
            } else {
                true
            };
            #[cfg(not(unix))]
            let handler_matched = true;

            let pidfile_matched = pid_from_pidfile.is_none_or(|p| p == pid.pid as i64);

            if (run_state_matched
                && pattern_matched
                && tty_matched
                && older_matched
                && parent_matched
                && pgroup_matched
                && session_matched
                && cgroup_matched
                && ids_matched
                && handler_matched
                && pidfile_matched)
                ^ settings.inverse
            {
                tmp_vec.push(pid);
            }
        }
        tmp_vec
    };

    Ok(filtered)
}

/// Sorting pids for flag `-o` and `-n`.
///
/// This function can also be used as a filter to filter out process information.
fn process_flag_o_n(
    settings: &Settings,
    pids: &mut [ProcessInformation],
) -> Vec<ProcessInformation> {
    if settings.oldest || settings.newest {
        pids.sort_by(|a, b| {
            b.clone()
                .start_time()
                .unwrap()
                .cmp(&a.clone().start_time().unwrap())
        });

        let start_time = if settings.newest {
            pids.first().cloned().unwrap().start_time().unwrap()
        } else {
            pids.last().cloned().unwrap().start_time().unwrap()
        };

        // There might be some process start at same time, so need to be filtered.
        let mut filtered = pids
            .iter()
            .filter(|it| (*it).clone().start_time().unwrap() == start_time)
            .collect::<Vec<_>>();

        if settings.newest {
            filtered.sort_by(|a, b| b.pid.cmp(&a.pid));
        } else {
            filtered.sort_by(|a, b| a.pid.cmp(&b.pid));
        }

        vec![filtered.first().cloned().unwrap().clone()]
    } else {
        pids.to_vec()
    }
}

#[cfg(unix)]
fn parse_signal_value(signal_name: &str) -> UResult<usize> {
    signal_by_name_or_value(signal_name)
        .ok_or_else(|| USimpleError::new(1, format!("Unknown signal {}", signal_name.quote())))
}

#[cfg(not(unix))]
pub fn usr2uid(_name: &str) -> io::Result<u32> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "unsupported on this platform",
    ))
}

#[cfg(not(unix))]
pub fn grp2gid(_name: &str) -> io::Result<u32> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "unsupported on this platform",
    ))
}

/// # Safety
///
/// Dummy implementation for unsupported platforms.
#[cfg(not(unix))]
pub unsafe fn getpgrp() -> u32 {
    panic!("unsupported on this platform");
}

/// # Safety
///
/// Dummy implementation for unsupported platforms.
#[cfg(not(unix))]
pub unsafe fn getsid(_pid: u32) -> u32 {
    panic!("unsupported on this platform");
}

fn parse_uid_or_username(uid_or_username: &str) -> io::Result<u32> {
    uid_or_username
        .parse::<u32>()
        .or_else(|_| usr2uid(uid_or_username))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid user name"))
}

fn parse_gid_or_group_name(gid_or_group_name: &str) -> io::Result<u32> {
    gid_or_group_name
        .parse::<u32>()
        .or_else(|_| grp2gid(gid_or_group_name))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid group name"))
}

pub fn parse_pidfile_content(content: &str) -> Option<i64> {
    let re = Regex::new(r"(?-m)^[[:blank:]]*(-?[0-9]+)(?:\s|$)").unwrap();
    re.captures(content)?.get(1)?.as_str().parse::<i64>().ok()
}

#[test]
fn test_parse_pidfile_content_valid() {
    assert_eq!(parse_pidfile_content(" 1234"), Some(1234));
    assert_eq!(parse_pidfile_content("-5678 "), Some(-5678));
    assert_eq!(parse_pidfile_content("   42\nfoo\n"), Some(42));
    assert_eq!(parse_pidfile_content("\t-99\tbar\n"), Some(-99));

    assert_eq!(parse_pidfile_content(""), None);
    assert_eq!(parse_pidfile_content("abc"), None);
    assert_eq!(parse_pidfile_content("0x42"), None);
    assert_eq!(parse_pidfile_content("2.3"), None);
    assert_eq!(parse_pidfile_content("\n123\n"), None);
}

pub fn read_pidfile(filename: &str) -> UResult<i64> {
    let content = fs::read_to_string(filename)
        .map_err(|e| USimpleError::new(1, format!("Failed to read pidfile {}: {}", filename, e)))?;

    let pid = parse_pidfile_content(&content)
        .ok_or_else(|| USimpleError::new(1, format!("Pidfile {} not valid", filename)))?;

    Ok(pid)
}

#[allow(clippy::cognitive_complexity)]
pub fn clap_args(pattern_help: &'static str, enable_v_flag: bool) -> Vec<Arg> {
    vec![
        if enable_v_flag {
            arg!(-v --inverse          "negates the matching").group("oldest_newest_inverse")
        } else {
            arg!(--inverse             "negates the matching").group("oldest_newest_inverse")
        },
        arg!(-H --"require-handler"    "match only if signal handler is present"),
        arg!(-c --count                "count of matching processes"),
        arg!(-f --full                 "use full process name to match"),
        arg!(-g --pgroup <PGID>        "match listed process group IDs")
            .value_delimiter(',')
            .value_parser(clap::value_parser!(u64)),
        arg!(-G --group <GID>          "match real group IDs")
            .value_delimiter(',')
            .value_parser(parse_gid_or_group_name),
        arg!(-i --"ignore-case"        "match case insensitively"),
        arg!(-n --newest               "select most recently started")
            .group("oldest_newest_inverse"),
        arg!(-o --oldest               "select least recently started")
            .group("oldest_newest_inverse"),
        arg!(-O --older <seconds>      "select where older than seconds")
            .value_parser(clap::value_parser!(u64)),
        arg!(-P --parent <PPID>        "match only child processes of the given parent")
            .value_delimiter(',')
            .value_parser(clap::value_parser!(u64)),
        arg!(-s --session <SID>        "match session IDs")
            .value_delimiter(',')
            .value_parser(clap::value_parser!(u64)),
        arg!(--signal <sig>            "signal to send (either number or name)")
            .default_value("SIGTERM"),
        arg!(-t --terminal <tty>       "match by controlling terminal").value_delimiter(','),
        arg!(-u --euid <ID>            "match by effective IDs")
            .value_delimiter(',')
            .value_parser(parse_uid_or_username),
        arg!(-U --uid <ID>             "match by real IDs")
            .value_delimiter(',')
            .value_parser(parse_uid_or_username),
        arg!(-x --exact                "match exactly with the command name"),
        arg!(-F --pidfile <file>       "read PIDs from file"),
        // arg!(-L --logpidfile           "fail if PID file is not locked"),
        arg!(-r --runstates <state>    "match runstates [D,S,Z,...]"),
        // arg!(-A --"ignore-ancestors"   "exclude our ancestors from results"),
        arg!(--cgroup <grp>            "match by cgroup v2 names").value_delimiter(','),
        // arg!(--ns <PID>                "match the processes that belong to the same namespace as <pid>"),
        // arg!(--nslist <ns>             "list which namespaces will be considered for the --ns option.")
        //     .value_delimiter(',')
        //     .value_parser(["ipc", "mnt", "net", "pid", "user", "uts"]),
        Arg::new("pattern")
            .help(pattern_help)
            .action(ArgAction::Append)
            .index(1),
    ]
}
