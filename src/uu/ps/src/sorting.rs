// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use uu_pgrep::process::ProcessInformation;

// TODO: Implementing sorting flags.
pub(crate) fn sort(input: &mut [ProcessInformation], _matches: &ArgMatches) {
    sort_by_pid(input);
}

/// Sort by pid. (Default)
fn sort_by_pid(input: &mut [ProcessInformation]) {
    input.sort_by(|a, b| a.pid.cmp(&b.pid));
}
