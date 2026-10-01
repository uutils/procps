#[cfg(target_os = "linux")]
use std::io;

/// See https://www.man7.org/linux/man-pages/man7/namespaces.7.html
///
/// # Support status
///
/// **_Linux only._**
#[derive(Default)]
pub struct Namespace {
    pub ipc: Option<u64>,
    pub mnt: Option<u64>,
    pub net: Option<u64>,
    pub pid: Option<u64>,
    pub user: Option<u64>,
    pub uts: Option<u64>,
}

impl Namespace {
    pub fn new() -> Self {
        Namespace {
            ipc: None,
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn from_pid(pid: usize) -> io::Result<Self> {
        use std::{os::fd::OwnedFd, path::PathBuf};

        use rustix::fs::{openat, statx, AtFlags, Mode, OFlags, StatxFlags, CWD};

        let f = |name: &str, fd: &OwnedFd| {
            statx(
                fd,
                name,
                AtFlags::empty(), // NO FOLLOW LINKS
                StatxFlags::INO,  // INNODE ONLY
            )
        };

        let ns_dir = openat(
            CWD,
            PathBuf::from(format!("/proc/{}/ns", pid)),
            OFlags::RDONLY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let mut ns = Namespace::default();

        for (name, slot) in [
            ("ipc", &mut ns.ipc),
            ("mnt", &mut ns.mnt),
            ("net", &mut ns.net),
            ("pid", &mut ns.pid),
            ("user", &mut ns.user),
            ("uts", &mut ns.uts),
        ] {
            let st = f(name, &ns_dir)?;
            *slot = Some(st.stx_ino);
        }
        Ok(ns)
    }

    /// TODO: implementation for other system
    #[cfg(not(target_os = "linux"))]
    pub fn from_pid(_pid: usize) -> Result<Self, io::Error> {
        Ok(Namespace::new())
    }

    pub fn filter(&mut self, filters: &[&str]) {
        if !filters.contains(&"ipc") {
            self.ipc = None;
        }
        if !filters.contains(&"mnt") {
            self.mnt = None;
        }
        if !filters.contains(&"net") {
            self.net = None;
        }
        if !filters.contains(&"pid") {
            self.pid = None;
        }
        if !filters.contains(&"user") {
            self.user = None;
        }
        if !filters.contains(&"uts") {
            self.uts = None;
        }
    }

    pub fn matches(&self, ns: &Namespace) -> bool {
        ns.ipc.is_some()
            && self
                .ipc
                .as_ref()
                .is_some_and(|v| v == ns.ipc.as_ref().unwrap())
            || ns.mnt.is_some()
                && self
                    .mnt
                    .as_ref()
                    .is_some_and(|v| v == ns.mnt.as_ref().unwrap())
            || ns.net.is_some()
                && self
                    .net
                    .as_ref()
                    .is_some_and(|v| v == ns.net.as_ref().unwrap())
            || ns.pid.is_some()
                && self
                    .pid
                    .as_ref()
                    .is_some_and(|v| v == ns.pid.as_ref().unwrap())
            || ns.user.is_some()
                && self
                    .user
                    .as_ref()
                    .is_some_and(|v| v == ns.user.as_ref().unwrap())
            || ns.uts.is_some()
                && self
                    .uts
                    .as_ref()
                    .is_some_and(|v| v == ns.uts.as_ref().unwrap())
    }
}
