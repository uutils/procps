use clap::ArgMatches;
use uu_pgrep::process::ProcessInformation;

/// Filter for processes
///
/// - `-A` (alias `-e`)
pub(crate) fn process_collector(
    matches: &ArgMatches,
    proc_snapshot: &[ProcessInformation],
) -> Vec<ProcessInformation> {
    let mut result = Vec::new();

    let basic_collector = (|| {})();

    let flag_A_collector = (|| {})();

    result
}

/// Filter for session
///
/// - `-d`
/// - `-a`
pub(crate) fn session_collector(
    matches: &ArgMatches,
    proc_snapshot: &[ProcessInformation],
) -> Vec<ProcessInformation> {
    let mut result = Vec::new();

    // session id
    // https://docs.kernel.org/filesystems/proc.html#id10
    let flag_d_collector = || {};
    let flag_a_collector = || {};

    result
}

/// Filter for terminal
///
/// - `-t`
pub(crate) fn terminal_filter(
    matches: &ArgMatches,
    proc_snapshot: &[ProcessInformation],
) -> Vec<ProcessInformation> {
    let mut result = Vec::new();

    let flag_a_collector = || {};

    result
}
