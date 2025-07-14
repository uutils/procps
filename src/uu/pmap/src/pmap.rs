// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, Command};
use maps_format_parser::{parse_map_line, MapLine};
use pmap_config::{create_rc, pmap_field_name, PmapConfig};
use smaps_format_parser::{parse_smaps, SmapTable};
use std::env;
use std::fs;
use std::io::Error;
use uucore::error::{set_exit_code, UResult};
use uucore::{format_usage, help_about, help_usage};

mod maps_format_parser;
mod pmap_config;
mod smaps_format_parser;

const ABOUT: &str = help_about!("pmap.md");
const USAGE: &str = help_usage!("pmap.md");

mod options {
    pub const PID: &str = "pid";
    pub const EXTENDED: &str = "extended";
    pub const MORE_EXTENDED: &str = "more-extended";
    pub const MOST_EXTENDED: &str = "most-extended";
    pub const READ_RC: &str = "read-rc";
    pub const READ_RC_FROM: &str = "read-rc-from";
    pub const CREATE_RC: &str = "create-rc";
    pub const CREATE_RC_TO: &str = "create-rc-to";
    pub const DEVICE: &str = "device";
    pub const QUIET: &str = "quiet";
    pub const SHOW_PATH: &str = "show-path";
    pub const RANGE: &str = "range";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    if matches.get_flag(options::CREATE_RC) {
        let path = pmap_config::get_rc_default_path();
        if std::fs::exists(&path)? {
            eprintln!("pmap: the file already exists - delete or rename it first");
            eprintln!(
                "pmap: couldn't create {}",
                pmap_config::get_rc_default_path_str()
            );
            set_exit_code(1);
        } else {
            create_rc(&path)?;
            eprintln!(
                "pmap: {} file successfully created, feel free to edit the content",
                pmap_config::get_rc_default_path_str()
            );
        }
        return Ok(());
    } else if let Some(path_str) = matches.get_one::<String>(options::CREATE_RC_TO) {
        let path = std::path::PathBuf::from(path_str);
        if std::fs::exists(&path)? {
            eprintln!("pmap: the file already exists - delete or rename it first");
            eprintln!("pmap: couldn't create the rc file");
            set_exit_code(1);
        } else {
            create_rc(&path)?;
            eprintln!("pmap: rc file successfully created, feel free to edit the content");
        }
        return Ok(());
    }

    let mut pmap_config = PmapConfig::default();

    if matches.get_flag(options::MORE_EXTENDED) {
        pmap_config.set_more_extended();
    } else if matches.get_flag(options::MOST_EXTENDED) {
        pmap_config.set_most_extended();
    } else if matches.get_flag(options::READ_RC) {
        let path = pmap_config::get_rc_default_path();
        if !std::fs::exists(&path)? {
            eprintln!(
                "pmap: couldn't read {}",
                pmap_config::get_rc_default_path_str()
            );
            set_exit_code(1);
            return Ok(());
        }
        pmap_config.read_rc(&path)?;
    } else if let Some(path) = matches.get_one::<String>(options::READ_RC_FROM) {
        let path = std::fs::canonicalize(path)?;
        if !std::fs::exists(&path)? {
            eprintln!("pmap: couldn't read the rc file");
            set_exit_code(1);
            return Ok(());
        }
        pmap_config.read_rc(&path)?;
    }

    // Options independent with field selection:
    pmap_config.quiet = matches.get_flag(options::QUIET);
    if matches.get_flag(options::SHOW_PATH) {
        pmap_config.show_path = true;
    }

    let pids = matches
        .get_many::<String>(options::PID)
        .expect("PID required");

    for pid in pids {
        match parse_cmdline(pid) {
            Ok(cmdline) => {
                println!("{pid}:   {cmdline}");
            }
            Err(_) => {
                set_exit_code(42);
                continue;
            }
        }

        if matches.get_flag(options::EXTENDED) {
            output_extended_format(pid, &pmap_config)
                .map_err(|_| set_exit_code(1))
                .ok();
        } else if matches.get_flag(options::DEVICE) {
            output_device_format(pid, &pmap_config)
                .map_err(|_| set_exit_code(1))
                .ok();
        } else if pmap_config.custom_format_enabled {
            output_custom_format(pid, &mut pmap_config)
                .map_err(|_| set_exit_code(1))
                .ok();
        } else {
            output_default_format(pid, &pmap_config)
                .map_err(|_| set_exit_code(1))
                .ok();
        }
    }

    Ok(())
}

fn parse_cmdline(pid: &str) -> Result<String, Error> {
    let path = format!("/proc/{pid}/cmdline");
    let contents = fs::read(path)?;
    // Command line arguments are separated by null bytes.
    // Replace them with spaces for display.
    let cmdline = contents
        .split(|&c| c == 0)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect::<Vec<&str>>()
        .join(" ");
    let cmdline = cmdline.trim_end();
    Ok(cmdline.into())
}

fn process_maps<F>(pid: &str, header: Option<&str>, mut process_line: F) -> Result<(), Error>
where
    F: FnMut(&MapLine),
{
    let path = format!("/proc/{pid}/maps");
    let contents = fs::read_to_string(path)?;

    if let Some(header) = header {
        println!("{header}");
    }

    for line in contents.lines() {
        let map_line = parse_map_line(line)?;
        process_line(&map_line);
    }

    Ok(())
}

fn get_smap_table(pid: &str) -> Result<SmapTable, Error> {
    let path = format!("/proc/{pid}/smaps");
    let contents = fs::read_to_string(path)?;
    parse_smaps(&contents)
}

fn output_default_format(pid: &str, pmap_config: &PmapConfig) -> Result<(), Error> {
    let mut total = 0;

    process_maps(pid, None, |map_line| {
        println!(
            "{} {:>6}K {} {}",
            map_line.address.zero_pad(),
            map_line.size_in_kb,
            map_line.perms.mode(),
            map_line.parse_mapping(pmap_config)
        );
        total += map_line.size_in_kb;
    })?;

    if !pmap_config.quiet {
        println!(" total {total:>16}K");
    }

    Ok(())
}

fn output_extended_format(pid: &str, pmap_config: &PmapConfig) -> Result<(), Error> {
    let smap_table = get_smap_table(pid)?;

    if !pmap_config.quiet {
        println!("Address           Kbytes     RSS   Dirty Mode  Mapping");
    }

    for smap_entry in smap_table.entries {
        println!(
            "{} {:>7} {:>7} {:>7} {} {}",
            smap_entry.map_line.address.zero_pad(),
            smap_entry.map_line.size_in_kb,
            smap_entry.rss_in_kb,
            smap_entry.shared_dirty_in_kb + smap_entry.private_dirty_in_kb,
            smap_entry.map_line.perms.mode(),
            smap_entry.map_line.parse_mapping(pmap_config)
        );
    }

    if !pmap_config.quiet {
        println!("---------------- ------- ------- ------- ");
        println!(
            "total kB         {:>7} {:>7} {:>7}",
            smap_table.info.total_size_in_kb,
            smap_table.info.total_rss_in_kb,
            smap_table.info.total_shared_dirty_in_kb + smap_table.info.total_private_dirty_in_kb,
        );
    }

    Ok(())
}

fn output_custom_format(pid: &str, pmap_config: &mut PmapConfig) -> Result<(), Error> {
    let smap_table = get_smap_table(pid)?;

    if !smap_table.info.has_ksm {
        pmap_config.disable_field(pmap_field_name::KSM);
    }
    if !smap_table.info.has_protection_key {
        pmap_config.disable_field(pmap_field_name::PROTECTION_KEY);
    }

    // Header
    if !pmap_config.quiet {
        let mut line = format!(
            "{:>width$} ",
            pmap_field_name::ADDRESS,
            width = smap_table.info.get_width(pmap_field_name::ADDRESS)
        );

        pmap_config.quiet = true;
        for field_name in pmap_config.get_field_list() {
            if pmap_config.is_enabled(field_name) {
                // If there is any field that needs footer, we can't suppress the footer
                if pmap_config.needs_footer(field_name) {
                    pmap_config.quiet = false;
                }
                line += &format!(
                    "{:>width$} ",
                    field_name,
                    width = smap_table.info.get_width(field_name)
                );
            }
        }
        if pmap_config.is_enabled(pmap_field_name::MAPPING) {
            line += pmap_field_name::MAPPING;
        }
        println!("{line}");
    }

    // Main
    for smap_entry in smap_table.entries {
        let mut line = format!(
            "{:>width$} ",
            smap_entry.get_field(pmap_field_name::ADDRESS),
            width = smap_table.info.get_width(pmap_field_name::ADDRESS)
        );
        for field_name in pmap_config.get_field_list() {
            if pmap_config.is_enabled(field_name) {
                line += &format!(
                    "{:>width$} ",
                    smap_entry.get_field(field_name),
                    width = smap_table.info.get_width(field_name)
                );
            }
        }
        if pmap_config.is_enabled(pmap_field_name::MAPPING) {
            line += &smap_entry.map_line.parse_mapping(pmap_config);
        }
        println!("{line}");
    }

    // Footer
    if !pmap_config.quiet {
        // Separator
        let mut line = format!(
            "{:>width$} ",
            "",
            width = smap_table.info.get_width(pmap_field_name::ADDRESS)
        );
        for field_name in pmap_config.get_field_list() {
            if pmap_config.is_enabled(field_name) && field_name != pmap_field_name::VMFLAGS {
                if pmap_config.needs_footer(field_name) {
                    line += &format!(
                        "{:=>width$} ",
                        "",
                        width = smap_table.info.get_width(field_name)
                    );
                } else {
                    line += &format!(
                        "{:>width$} ",
                        "",
                        width = smap_table.info.get_width(field_name)
                    );
                }
            }
        }
        println!("{line}");

        // Total values
        let mut line = format!(
            "{:>width$} ",
            "",
            width = smap_table.info.get_width(pmap_field_name::ADDRESS)
        );
        for field_name in pmap_config.get_field_list() {
            if pmap_config.is_enabled(field_name) && field_name != pmap_field_name::VMFLAGS {
                if pmap_config.needs_footer(field_name) {
                    line += &format!(
                        "{:>width$} ",
                        smap_table.info.get_total(field_name),
                        width = smap_table.info.get_width(field_name)
                    );
                } else {
                    line += &format!(
                        "{:>width$} ",
                        "",
                        width = smap_table.info.get_width(field_name)
                    );
                }
            }
        }
        println!("{line}KB ");
    }

    Ok(())
}

fn output_device_format(pid: &str, pmap_config: &PmapConfig) -> Result<(), Error> {
    let mut total_mapped = 0;
    let mut total_writeable_private = 0;
    let mut total_shared = 0;

    process_maps(
        pid,
        if !pmap_config.quiet {
            Some("Address           Kbytes Mode  Offset           Device    Mapping")
        } else {
            None
        },
        |map_line| {
            println!(
                "{} {:>7} {} {:0>16} {} {}",
                map_line.address.zero_pad(),
                map_line.size_in_kb,
                map_line.perms.mode(),
                map_line.offset,
                map_line.device.device(),
                map_line.parse_mapping(pmap_config)
            );
            total_mapped += map_line.size_in_kb;

            if map_line.perms.writable && !map_line.perms.shared {
                total_writeable_private += map_line.size_in_kb;
            }

            if map_line.perms.shared {
                total_shared += map_line.size_in_kb;
            }
        },
    )?;

    if !pmap_config.quiet {
        println!(
            "mapped: {total_mapped}K    writeable/private: {total_writeable_private}K    shared: {total_shared}K"
        );
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new(options::PID)
                .help("Process ID")
                .required_unless_present_any(["create-rc", "create-rc-to"]) // Adjusted for -n, -N note
                .action(ArgAction::Append)
                .conflicts_with_all(["create-rc", "create-rc-to"]),
        ) // Ensure pid is not used with -n, -N
        .arg(
            Arg::new(options::EXTENDED)
                .short('x')
                .long("extended")
                .help("show details")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "device",
                    "create-rc",
                    "create-rc-to",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::MORE_EXTENDED)
                .short('X')
                .help("show even more details")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "device",
                    "create-rc",
                    "create-rc-to",
                    "extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::MOST_EXTENDED)
                .long("XX")
                .help("show everything the kernel provides")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "device",
                    "create-rc",
                    "create-rc-to",
                    "extended",
                    "more-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::READ_RC)
                .short('c')
                .long("read-rc")
                .help("read the default rc")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc-from",
                    "device",
                    "create-rc",
                    "create-rc-to",
                    "extended",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::READ_RC_FROM)
                .short('C')
                .long("read-rc-from")
                .num_args(1)
                .help("read the rc from file")
                .conflicts_with_all([
                    "read-rc",
                    "device",
                    "create-rc",
                    "create-rc-to",
                    "extended",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::CREATE_RC)
                .short('n')
                .long("create-rc")
                .help("create new default rc")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "device",
                    "create-rc-to",
                    "extended",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::CREATE_RC_TO)
                .short('N')
                .long("create-rc-to")
                .num_args(1)
                .help("create new rc to file")
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "device",
                    "create-rc",
                    "extended",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::DEVICE)
                .short('d')
                .long("device")
                .help("show the device format")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "read-rc",
                    "read-rc-from",
                    "create-rc",
                    "create-rc-to",
                    "extended",
                    "more-extended",
                    "most-extended",
                ]),
        ) // pmap: options -c, -C, -d, -n, -N, -x, -X are mutually exclusive
        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .long("quiet")
                .help("do not display header and footer")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::SHOW_PATH)
                .short('p')
                .long("show-path")
                .help("show path in the mapping")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::RANGE)
                .short('A')
                .long("range")
                .num_args(1..=2)
                .help("limit results to the given range"),
        )
}
