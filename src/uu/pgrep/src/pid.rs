// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{collections::HashMap, fs, io, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct PidEntry {
    pub pid: usize,
    pub cmdline: String,
    pub status: HashMap<String, String>,
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
        let status = {
            let content =
                fs::read_to_string(dir_append(value.clone().into_path(), "status".into()))?;

            content
                .lines()
                .filter_map(|it| it.split_once(':'))
                .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
                .collect::<HashMap<_, _>>()
        };
        Ok(Self {
            pid,
            cmdline,
            status,
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
