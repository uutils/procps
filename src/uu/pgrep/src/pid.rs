// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{collections::HashMap, fs, io, path::PathBuf, rc::Rc};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Default)]
pub struct PidEntry {
    pub pid: usize,
    pub cmdline: String,

    inner_status: String,
    inner_stat: String,

    cached_status: Option<Rc<HashMap<String, String>>>,
    cached_stat: Option<Rc<Vec<String>>>,
}

impl PidEntry {
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
        // Kernel doc: https://docs.kernel.org/filesystems/proc.html#process-specific-subdirectories
        // Table 1-4
        Ok(self
            .stat()?
            .get(21)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u64>()
            .map_err(|_| io::ErrorKind::InvalidData)?)
    }
}

impl TryFrom<DirEntry> for PidEntry {
    type Error = io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let dir_append = |mut path: PathBuf, str: String| {
            path.push(str);
            path
        };

        let pid = {
            value
                .path()
                .iter()
                .last()
                .ok_or(io::ErrorKind::Other)?
                .to_str()
                .ok_or(io::ErrorKind::InvalidData)?
                .parse::<usize>()
                .map_err(|_| io::ErrorKind::InvalidData)?
        };
        let cmdline = fs::read_to_string(dir_append(value.clone().into_path(), "cmdline".into()))?
            .replace('\0', " ")
            .trim_end()
            .into();

        Ok(Self {
            pid,
            cmdline,
            inner_status: fs::read_to_string(dir_append(
                value.clone().into_path(),
                "status".into(),
            ))?,
            inner_stat: fs::read_to_string(dir_append(value.clone().into_path(), "stat".into()))?,
            ..Default::default()
        })
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
