// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
#[macro_use]
mod common;

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
