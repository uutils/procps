use std::sync::LazyLock;

use regex::Regex;
use walkdir::WalkDir;

use crate::process::ProcessInformation;

/// Iterating pid in current system
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    WalkDir::new("/proc/")
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|it| it.path().is_dir())
        .flat_map(ProcessInformation::try_from)
}

pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    static THREAD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^/proc/[0-9]+$|^/proc/[0-9]+/task$|^/proc/[0-9]+/task/[0-9]+$").unwrap()
    });

    WalkDir::new("/proc/")
        .min_depth(1)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| THREAD_REGEX.is_match(e.path().as_os_str().to_string_lossy().as_ref()))
        .flatten()
        .filter(|it| it.path().as_os_str().to_string_lossy().contains("/task/"))
        .flat_map(ProcessInformation::try_from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_walk_pid() {
        use rustix::process::getpid;

        let find = walk_process().find(|it| it.pid == getpid().as_raw_pid() as usize);

        assert!(find.is_some());
    }
}
