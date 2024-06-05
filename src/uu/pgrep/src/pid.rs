// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::PathBuf,
    rc::Rc,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalType {
    Tty(u64),
    TtyS(u64),
    Pts(u64),
}

impl TryFrom<String> for TerminalType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(value))
    }
}

impl TryFrom<&str> for TerminalType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(value))
    }
}

impl TryFrom<PathBuf> for TerminalType {
    type Error = ();

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        // Three case: /dev/pts/* , /dev/ttyS**, /dev/tty**

        let mut iter = value.iter();
        // Case 1

        // Considering this format: **/**/pts/<num>
        if let (Some(_), Some(num)) = (iter.find(|it| *it == "pts"), iter.next()) {
            return num
                .to_str()
                .ok_or(())?
                .parse::<u64>()
                .map_err(|_| ())
                .map(TerminalType::Pts);
        };

        // Considering this format: **/**/ttyS** then **/**/tty**
        let path = value.to_str().ok_or(())?;

        let f = |prefix: &str| {
            value
                .iter()
                .last()?
                .to_str()?
                .strip_prefix(prefix)?
                .parse::<u64>()
                .ok()
        };

        if path.contains("ttyS") {
            // Case 2
            f("ttyS").ok_or(()).map(TerminalType::TtyS)
        } else if path.contains("tty") {
            // Case 3
            f("tty").ok_or(()).map(TerminalType::Tty)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PidEntry {
    pub pid: usize,
    pub cmdline: String,

    inner_status: String,
    inner_stat: String,

    cached_status: Option<Rc<HashMap<String, String>>>,
    cached_stat: Option<Rc<Vec<String>>>,

    cached_start_time: Option<u64>,
    cached_tty: Option<Rc<HashSet<TerminalType>>>,
}

impl PidEntry {
    pub fn try_new(value: PathBuf) -> Result<Self, io::Error> {
        let dir_append = |mut path: PathBuf, str: String| {
            path.push(str);
            path
        };

        let value = if value.is_symlink() {
            fs::read_link(value)?
        } else {
            value
        };

        let pid = {
            value
                .iter()
                .last()
                .ok_or(io::ErrorKind::Other)?
                .to_str()
                .ok_or(io::ErrorKind::InvalidData)?
                .parse::<usize>()
                .map_err(|_| io::ErrorKind::InvalidData)?
        };
        let cmdline = fs::read_to_string(dir_append(value.clone(), "cmdline".into()))?
            .replace('\0', " ")
            .trim_end()
            .into();

        Ok(Self {
            pid,
            cmdline,
            inner_status: fs::read_to_string(dir_append(value.clone(), "status".into()))?,
            inner_stat: fs::read_to_string(dir_append(value.clone(), "stat".into()))?,
            ..Default::default()
        })
    }

    pub fn status(&mut self) -> Rc<HashMap<String, String>> {
        if let Some(c) = &self.cached_status {
            return Rc::clone(c);
        }

        let result = self
            .inner_status
            .lines()
            .filter_map(|it| it.split_once(':'))
            .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
            .collect::<HashMap<_, _>>();

        let result = Rc::new(result);
        self.cached_status = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    fn stat(&mut self) -> Result<Rc<Vec<String>>, io::Error> {
        if let Some(c) = &self.cached_stat {
            return Ok(Rc::clone(c));
        }

        let result: Vec<_> = {
            let mut buf = String::with_capacity(self.inner_stat.len());

            let l = self.inner_stat.find('(');
            let r = self.inner_stat.find(')');
            let content = if let (Some(l), Some(r)) = (l, r) {
                let replaced = self.inner_stat[(l + 1)..r].replace(' ', "$$");

                buf.push_str(&self.inner_stat[..l]);
                buf.push_str(&replaced);
                buf.push_str(&self.inner_stat[(r + 1)..self.inner_stat.len()]);

                &buf
            } else {
                &self.inner_stat
            };

            content
                .split_whitespace()
                .map(|it| it.replace("$$", " "))
                .collect()
        };

        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Ok(Rc::clone(&result))
    }

    pub fn start_time(&mut self) -> Result<u64, io::Error> {
        if let Some(time) = self.cached_start_time {
            return Ok(time);
        }

        // Kernel doc: https://docs.kernel.org/filesystems/proc.html#process-specific-subdirectories
        // Table 1-4
        let time = self
            .stat()?
            .get(21)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u64>()
            .map_err(|_| io::ErrorKind::InvalidData)?;

        Ok(time)
    }

    /// This function will scan the `/proc/<pid>/df` directory
    ///
    /// # Error
    ///
    /// If scanned pid undering mismatched permission,
    /// it will caused [std::io::ErrorKind::PermissionDenied] error.
    pub fn ttys(&mut self) -> Result<Rc<HashSet<TerminalType>>, io::Error> {
        if let Some(tty) = &self.cached_tty {
            return Ok(Rc::clone(tty));
        }

        let path = PathBuf::from(format!("/proc/{}/fd", self.pid));

        let result = Rc::new(
            fs::read_dir(path)?
                .flatten()
                .filter(|it| it.path().is_symlink())
                .flat_map(|it| fs::read_link(it.path()))
                .flat_map(TerminalType::try_from)
                .collect::<HashSet<_>>(),
        );

        self.cached_tty = Some(Rc::clone(&result));

        Ok(result)
    }
}

impl TryFrom<DirEntry> for PidEntry {
    type Error = io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let value = value.into_path();

        PidEntry::try_new(value)
    }
}

pub fn walk_pid() -> impl Iterator<Item = PidEntry> {
    WalkDir::new("/proc/")
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|it| it.path().is_dir())
        .flat_map(PidEntry::try_from)
}
