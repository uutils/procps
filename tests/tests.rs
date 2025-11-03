// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
use std::env;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_procps");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor]
fn init() {
    unsafe {
        // Necessary for uutests to be able to find the binary
        std::env::set_var("UUTESTS_BINARY_PATH", TESTS_BINARY);
    }
}

#[cfg(feature = "pwdx")]
#[path = "by-util/test_pwdx.rs"]
mod test_pwdx;

#[cfg(feature = "free")]
#[path = "by-util/test_free.rs"]
mod test_free;

#[cfg(feature = "w")]
#[path = "by-util/test_w.rs"]
mod test_w;

#[cfg(feature = "watch")]
#[path = "by-util/test_watch.rs"]
mod test_watch;

#[cfg(feature = "pmap")]
#[path = "by-util/test_pmap.rs"]
mod test_pmap;

#[cfg(feature = "slabtop")]
#[path = "by-util/test_slabtop.rs"]
mod test_slabtop;

#[cfg(feature = "pgrep")]
#[path = "by-util/test_pgrep.rs"]
mod test_pgrep;

#[cfg(feature = "pidof")]
#[path = "by-util/test_pidof.rs"]
mod test_pidof;

#[cfg(feature = "ps")]
#[path = "by-util/test_ps.rs"]
mod test_ps;

#[cfg(feature = "pidwait")]
#[path = "by-util/test_pidwait.rs"]
mod test_pidwait;

#[cfg(feature = "top")]
#[path = "by-util/test_top.rs"]
mod test_top;

#[cfg(feature = "vmstat")]
#[path = "by-util/test_vmstat.rs"]
mod test_vmstat;

#[cfg(feature = "skill")]
#[path = "by-util/test_skill.rs"]
mod test_skill;

#[cfg(feature = "snice")]
#[path = "by-util/test_snice.rs"]
mod test_snice;

#[cfg(feature = "pkill")]
#[path = "by-util/test_pkill.rs"]
mod test_pkill;

#[cfg(feature = "sysctl")]
#[path = "by-util/test_sysctl.rs"]
mod test_sysctl;

#[cfg(feature = "tload")]
#[path = "by-util/test_tload.rs"]
mod test_tload;

#[path = "by-util/test_uuproc.rs"]
mod test_uuproc;
