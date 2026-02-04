// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Common process matcher logic shared by pgrep, pkill and pidwait

use std::hash::Hash;
#[cfg(unix)]
use std::os::fd::AsRawFd;
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

use crate::process::{walk_process, walk_threads, Namespace, ProcessInformation, Teletype};

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
    pub namespaces: Option<Namespace>,
    pub env: Option<HashSet<String>>,
    pub threads: bool,

    pub pidfile: Option<String>,
    pub logpidfile: bool,
    pub ignore_ancestors: bool,
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
        namespaces: matches
            .get_one::<usize>("ns")
            .map(|pid| {
                get_namespaces(
                    *pid,
                    matches
                        .get_many::<String>("nslist")
                        .map(|v| v.into_iter().map(|s| s.as_str()).collect()),
                )
            })
            .transpose()?,
        env: matches
            .get_many::<String>("env")
            .map(|env_vars| env_vars.cloned().collect()),
        threads: false,
        pidfile: matches.get_one::<String>("pidfile").cloned(),
        logpidfile: matches.get_flag("logpidfile"),
        ignore_ancestors: matches.get_flag("ignore-ancestors"),
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
        && settings.namespaces.is_none()
        && settings.env.is_none()
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

    // Pre-collect pids to check for possible match due to long pattern
    let pids = collect_matched_pids(&settings)?;
    let mut matches = false;
    if !pids.is_empty() {
        matches = true;
    }

    if !matches && !settings.full && pattern.len() > 15 {
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
        &format!("^{pattern}$")
    } else {
        pattern
    };

    Ok(pattern.clone())
}

fn any_matches<T: Eq + Hash>(optional_ids: &Option<HashSet<T>>, id: T) -> bool {
    optional_ids.as_ref().is_none_or(|ids| ids.contains(&id))
}

fn get_ancestors(process_infos: &mut [ProcessInformation], mut pid: usize) -> HashSet<usize> {
    let mut ret = HashSet::from([pid]);
    while pid != 1 {
        if let Some(process) = process_infos.iter_mut().find(|p| p.pid == pid) {
            pid = process.ppid().unwrap() as usize;
            ret.insert(pid);
        } else {
            break;
        }
    }
    ret
}

#[cfg(target_os = "linux")]
fn get_namespaces(pid: usize, list: Option<Vec<&str>>) -> UResult<Namespace> {
    let mut ns = Namespace::from_pid(pid)
        .map_err(|_| USimpleError::new(1, "Error reading reference namespace information"))?;
    if let Some(list) = list {
        ns.filter(&list);
    }

    Ok(ns)
}

#[cfg(not(target_os = "linux"))]
fn get_namespaces(_pid: usize, _list: Option<Vec<&str>>) -> UResult<Namespace> {
    Ok(Namespace::new())
}

/// Collect pids with filter construct from command line arguments
fn collect_matched_pids(settings: &Settings) -> UResult<Vec<ProcessInformation>> {
    // Filtration general parameters
    let filtered: Vec<ProcessInformation> = {
        let mut tmp_vec = Vec::new();

        let mut pids = if settings.threads {
            walk_threads().collect::<Vec<_>>()
        } else {
            walk_process().collect::<Vec<_>>()
        };
        let our_pid = std::process::id() as usize;
        let ignored_pids = if settings.ignore_ancestors {
            get_ancestors(&mut pids, our_pid)
        } else {
            HashSet::from([our_pid])
        };

        let pid_from_pidfile = settings
            .pidfile
            .as_ref()
            .map(|filename| read_pidfile(filename, settings.logpidfile))
            .transpose()?;

        for mut pid in pids {
            if ignored_pids.contains(&pid.pid) {
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
            let namespace_matched = settings
                .namespaces
                .as_ref()
                .is_none_or(|ns| ns.matches(&pid.namespaces().unwrap_or_default()));

            let env_matched = match &settings.env {
                Some(env_filters) => {
                    let env_vars = pid.env_vars().unwrap_or_default();
                    env_filters.iter().any(|filter| {
                        if let Some((key, expected_value)) = filter.split_once('=') {
                            // Match specific key=value pair
                            env_vars.get(key) == Some(&expected_value.to_string())
                        } else {
                            // Match key existence only
                            env_vars.contains_key(filter)
                        }
                    })
                }
                None => true,
            };

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
                let mask = pid.clone().signals_caught_mask().unwrap();
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
                && namespace_matched
                && env_matched
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

fn parse_pidfile_content(content: &str) -> Option<i64> {
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

#[cfg(unix)]
fn is_locked(file: &std::fs::File) -> bool {
    // On Linux, fcntl and flock locks are independent, so need to check both
    let mut flock_struct = uucore::libc::flock {
        l_type: uucore::libc::F_RDLCK as uucore::libc::c_short,
        l_whence: uucore::libc::SEEK_SET as uucore::libc::c_short,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    let fd = file.as_raw_fd();
    let result = unsafe { uucore::libc::fcntl(fd, uucore::libc::F_GETLK, &mut flock_struct) };
    if result == 0 && flock_struct.l_type != uucore::libc::F_UNLCK as uucore::libc::c_short {
        return true;
    }

    let result = unsafe { uucore::libc::flock(fd, uucore::libc::LOCK_SH | uucore::libc::LOCK_NB) };
    if result == -1 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::WouldBlock {
        return true;
    }

    false
}

#[cfg(not(unix))]
fn is_locked(_file: &std::fs::File) -> bool {
    // Dummy implementation just to make it compile
    false
}

fn read_pidfile(filename: &str, check_locked: bool) -> UResult<i64> {
    let file = std::fs::File::open(filename)
        .map_err(|e| USimpleError::new(1, format!("Failed to open pidfile {filename}: {e}")))?;

    if check_locked && !is_locked(&file) {
        return Err(USimpleError::new(
            1,
            format!("Pidfile {filename} is not locked"),
        ));
    }

    let content = std::fs::read_to_string(filename)
        .map_err(|e| USimpleError::new(1, format!("Failed to read pidfile {filename}: {e}")))?;

    let pid = parse_pidfile_content(&content)
        .ok_or_else(|| USimpleError::new(1, format!("Pidfile {filename} not valid")))?;

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
        arg!(-L --logpidfile           "fail if PID file is not locked"),
        arg!(-r --runstates <state>    "match runstates [D,S,Z,...]"),
        arg!(-A --"ignore-ancestors"   "exclude our ancestors from results"),
        arg!(--cgroup <grp>            "match by cgroup v2 names").value_delimiter(','),
        arg!(--env <"name[=val],...">      "match on environment variable").value_delimiter(','),
        arg!(--ns <PID>                "match the processes that belong to the same namespace as <pid>")
            .value_parser(clap::value_parser!(usize)),
        arg!(--nslist <ns>             "list which namespaces will be considered for the --ns option.")
            .value_delimiter(',')
            .value_parser(["ipc", "mnt", "net", "pid", "user", "uts"]),
        Arg::new("pattern")
            .help(pattern_help)
            .action(ArgAction::Append)
            .index(1),
    ]
}
