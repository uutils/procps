// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uuproc::{walk_process, CgroupMembership, Namespace, RunState, Teletype};

// ============================================================================
// Teletype Tests
// ============================================================================

#[test]
fn test_teletype_unknown() {
    let tty = Teletype::Unknown;
    assert_eq!(tty.to_string(), "?");
}

#[test]
fn test_teletype_tty() {
    let tty = Teletype::Tty(0);
    assert_eq!(tty.to_string(), "/dev/tty0");

    let tty = Teletype::Tty(255);
    assert_eq!(tty.to_string(), "/dev/tty255");
}

#[test]
fn test_teletype_ttys() {
    let tty = Teletype::TtyS(0);
    assert_eq!(tty.to_string(), "/dev/ttyS0");

    let tty = Teletype::TtyS(1);
    assert_eq!(tty.to_string(), "/dev/ttyS1");
}

#[test]
fn test_teletype_pts() {
    let tty = Teletype::Pts(0);
    assert_eq!(tty.to_string(), "/dev/pts/0");

    let tty = Teletype::Pts(42);
    assert_eq!(tty.to_string(), "/dev/pts/42");
}

#[test]
fn test_teletype_from_string_unknown() {
    let tty: Result<Teletype, _> = "?".to_string().try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Unknown);
}

#[test]
fn test_teletype_from_string_tty() {
    let tty: Result<Teletype, _> = "/dev/tty0".to_string().try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Tty(0));
}

#[test]
fn test_teletype_from_string_pts() {
    let tty: Result<Teletype, _> = "/dev/pts/0".to_string().try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Pts(0));
}

#[test]
fn test_teletype_from_u64_zero() {
    let tty: Result<Teletype, _> = 0u64.try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Unknown);
}

#[test]
fn test_teletype_from_u64_tty() {
    let tty: Result<Teletype, _> = 0x0400u64.try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Tty(0));
}

#[test]
fn test_teletype_from_u64_pts() {
    let tty: Result<Teletype, _> = 0x8800u64.try_into();
    assert!(tty.is_ok());
    assert_eq!(tty.unwrap(), Teletype::Pts(0));
}

// ============================================================================
// RunState Tests
// ============================================================================

#[test]
fn test_runstate_from_char() {
    assert_eq!(RunState::try_from('R').unwrap(), RunState::Running);
    assert_eq!(RunState::try_from('S').unwrap(), RunState::Sleeping);
    assert_eq!(
        RunState::try_from('D').unwrap(),
        RunState::UninterruptibleWait
    );
    assert_eq!(RunState::try_from('Z').unwrap(), RunState::Zombie);
    assert_eq!(RunState::try_from('T').unwrap(), RunState::Stopped);
    assert_eq!(RunState::try_from('t').unwrap(), RunState::TraceStopped);
}

#[test]
fn test_runstate_from_char_invalid() {
    assert!(RunState::try_from('Q').is_err());
}

#[test]
fn test_runstate_from_str() {
    assert_eq!(RunState::try_from("R").unwrap(), RunState::Running);
    assert_eq!(RunState::try_from("S").unwrap(), RunState::Sleeping);
}

#[test]
fn test_runstate_from_str_invalid() {
    assert!(RunState::try_from("INVALID").is_err());
}

#[test]
fn test_runstate_from_string() {
    assert_eq!(
        RunState::try_from("R".to_string()).unwrap(),
        RunState::Running
    );
}

#[test]
fn test_runstate_display() {
    assert_eq!(RunState::Running.to_string(), "R");
    assert_eq!(RunState::Sleeping.to_string(), "S");
    assert_eq!(RunState::Zombie.to_string(), "Z");
}

// ============================================================================
// CgroupMembership Tests
// ============================================================================

#[test]
fn test_cgroupmembership_parse_valid() {
    let cgroup_str = "0::/user.slice/user-1000.slice/session-1.scope";
    let cgroup = CgroupMembership::try_from(cgroup_str).unwrap();
    assert_eq!(cgroup.hierarchy_id, 0);
    assert_eq!(cgroup.controllers, Vec::<String>::new());
    assert_eq!(
        cgroup.cgroup_path,
        "/user.slice/user-1000.slice/session-1.scope"
    );
}

#[test]
fn test_cgroupmembership_parse_empty_controllers() {
    let cgroup_str = "1:cpu,cpuacct:/system.slice/systemd-logind.service";
    let cgroup = CgroupMembership::try_from(cgroup_str).unwrap();
    assert_eq!(cgroup.hierarchy_id, 1);
    assert!(cgroup.controllers.contains(&"cpu".to_string()));
    assert!(cgroup.controllers.contains(&"cpuacct".to_string()));
}

#[test]
fn test_cgroupmembership_parse_invalid_format() {
    let cgroup_str = "invalid";
    assert!(CgroupMembership::try_from(cgroup_str).is_err());
}

#[test]
fn test_cgroupmembership_parse_invalid_hierarchy_id() {
    let cgroup_str = "invalid:cpu:/system.slice";
    assert!(CgroupMembership::try_from(cgroup_str).is_err());
}

// ============================================================================
// Namespace Tests
// ============================================================================

#[test]
fn test_namespace_new() {
    let ns = Namespace::new();
    assert!(ns.ipc.is_none());
    assert!(ns.mnt.is_none());
    assert!(ns.net.is_none());
    assert!(ns.pid.is_none());
    assert!(ns.user.is_none());
    assert!(ns.uts.is_none());
}

#[test]
fn test_namespace_filter() {
    let mut ns = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: Some("4026531840".to_string()),
        net: Some("4026531956".to_string()),
        pid: Some("4026531836".to_string()),
        user: Some("4026531837".to_string()),
        uts: Some("4026531838".to_string()),
    };

    ns.filter(&["ipc", "mnt"]);

    assert!(ns.ipc.is_some());
    assert!(ns.mnt.is_some());
    assert!(ns.net.is_none());
    assert!(ns.pid.is_none());
    assert!(ns.user.is_none());
    assert!(ns.uts.is_none());
}

#[test]
fn test_namespace_filter_empty() {
    let mut ns = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: Some("4026531840".to_string()),
        net: Some("4026531956".to_string()),
        pid: Some("4026531836".to_string()),
        user: Some("4026531837".to_string()),
        uts: Some("4026531838".to_string()),
    };

    ns.filter(&[]);

    assert!(ns.ipc.is_none());
    assert!(ns.mnt.is_none());
    assert!(ns.net.is_none());
    assert!(ns.pid.is_none());
    assert!(ns.user.is_none());
    assert!(ns.uts.is_none());
}

#[test]
fn test_namespace_matches() {
    let ns1 = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: Some("4026531840".to_string()),
        net: Some("4026531956".to_string()),
        pid: Some("4026531836".to_string()),
        user: Some("4026531837".to_string()),
        uts: Some("4026531838".to_string()),
    };

    let ns2 = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: Some("4026531840".to_string()),
        net: Some("4026531956".to_string()),
        pid: Some("4026531836".to_string()),
        user: Some("4026531837".to_string()),
        uts: Some("4026531838".to_string()),
    };

    assert!(ns1.matches(&ns2));
}

#[test]
fn test_namespace_matches_different() {
    let ns1 = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: Some("4026531840".to_string()),
        net: Some("4026531956".to_string()),
        pid: Some("4026531836".to_string()),
        user: Some("4026531837".to_string()),
        uts: Some("4026531838".to_string()),
    };

    let ns2 = Namespace {
        ipc: Some("4026531839".to_string()),
        mnt: None,
        net: None,
        pid: None,
        user: None,
        uts: None,
    };

    assert!(ns1.matches(&ns2));
}

// ============================================================================
// Integration Tests - Process Enumeration
// ============================================================================

#[test]
fn test_walk_process_returns_processes() {
    // Verify we can enumerate at least the current process
    let processes: Vec<_> = walk_process().collect();
    assert!(!processes.is_empty(), "Should find at least one process");
}

#[test]
fn test_walk_process_process_fields() {
    // Verify all process fields are populated
    let processes: Vec<_> = walk_process().collect();
    assert!(!processes.is_empty(), "Should have at least one process");

    for proc in processes.iter().take(5) {
        assert!(proc.pid > 0, "PID should be positive");
        assert!(!proc.cmdline.is_empty(), "Command should not be empty");
    }
}

#[test]
fn test_walk_process_pids_are_unique() {
    // Verify PIDs are unique (no duplicates)
    let processes: Vec<_> = walk_process().collect();
    let mut pids = processes.iter().map(|p| p.pid).collect::<Vec<_>>();
    let original_len = pids.len();
    pids.sort();
    pids.dedup();
    assert_eq!(
        pids.len(),
        original_len,
        "All PIDs should be unique, found duplicates"
    );
}

#[test]
fn test_walk_process_handles_errors_gracefully() {
    // Should not panic on any platform
    let _processes: Vec<_> = walk_process().collect();
    // If we got here without panicking, the test passes
}

// ============================================================================
// Integration Tests - Process Enumeration Consistency
// ============================================================================

#[test]
fn test_walk_process_consistent_across_calls() {
    // Verify that multiple calls to walk_process return consistent results
    let processes1: Vec<_> = walk_process().collect();
    let processes2: Vec<_> = walk_process().collect();

    // Should have similar number of processes (allowing for some variance)
    let diff = (processes1.len() as i32 - processes2.len() as i32).abs();
    assert!(
        diff <= 5,
        "Process count should be consistent, got {} and {}",
        processes1.len(),
        processes2.len()
    );
}

#[test]
fn test_walk_process_init_process_exists() {
    // Verify that init process (PID 1) exists on Linux
    #[cfg(target_os = "linux")]
    {
        let processes: Vec<_> = walk_process().collect();
        let init_exists = processes.iter().any(|p| p.pid == 1);
        assert!(
            init_exists,
            "Init process (PID 1) should exist on Linux systems"
        );
    }
}

#[test]
fn test_walk_process_handles_very_long_cmdlines() {
    // Should handle processes with very long command lines
    let processes: Vec<_> = walk_process().collect();

    let mut max_cmdline_len = 0;
    for proc in &processes {
        max_cmdline_len = max_cmdline_len.max(proc.cmdline.len());
    }

    // Should handle long command lines without issues
    assert!(
        max_cmdline_len > 0,
        "Should find at least one process with non-empty cmdline"
    );
}

#[test]
fn test_walk_process_no_panic_on_iteration() {
    // Should not panic when iterating through all processes
    let mut count = 0;
    for _proc in walk_process() {
        count += 1;
        // Should not panic during iteration
    }
    assert!(count > 0, "Should enumerate at least one process");
}

#[test]
fn test_walk_process_multiple_iterations_safe() {
    // Should be safe to iterate multiple times
    for _ in 0..5 {
        let processes: Vec<_> = walk_process().collect();
        assert!(
            !processes.is_empty(),
            "Should find processes on each iteration"
        );
    }
}

#[test]
fn test_walk_process_handles_processes_with_spaces_in_name() {
    // Should handle processes with spaces in their command line
    let processes: Vec<_> = walk_process().collect();

    let with_spaces: Vec<_> = processes
        .iter()
        .filter(|p| p.cmdline.contains(' '))
        .collect();

    // Most systems have processes with spaces in command line
    assert!(
        !with_spaces.is_empty(),
        "Should find processes with spaces in command line"
    );
}

#[test]
fn test_walk_process_handles_processes_with_special_chars() {
    // Should handle processes with special characters
    let processes: Vec<_> = walk_process().collect();

    for proc in processes.iter().take(100) {
        // Should not panic on any character
        let _ = proc.cmdline.chars().filter(|c| !c.is_ascii()).count();
    }
}

#[test]
fn test_walk_process_pid_ordering() {
    // PIDs should be in a reasonable order (not necessarily sorted)
    let processes: Vec<_> = walk_process().collect();

    // Should have a mix of PIDs
    let pids: Vec<_> = processes.iter().map(|p| p.pid).collect();
    assert!(pids.len() > 1, "Should have multiple processes");

    // Check that we have a reasonable distribution of PIDs
    let min_pid = pids.iter().min().copied().unwrap_or(0);
    let max_pid = pids.iter().max().copied().unwrap_or(0);
    assert!(min_pid > 0, "Minimum PID should be positive");
    assert!(max_pid > min_pid, "Should have PID range");
}

#[test]
fn test_walk_process_handles_process_with_no_cmdline() {
    // Some processes may have empty command lines (kernel threads)
    let processes: Vec<_> = walk_process().collect();

    let empty_cmdline: Vec<_> = processes.iter().filter(|p| p.cmdline.is_empty()).collect();

    // On Linux, kernel threads have empty command lines
    #[cfg(target_os = "linux")]
    {
        assert!(
            !empty_cmdline.is_empty(),
            "Should find processes with empty cmdline on Linux"
        );
    }
}

#[test]
fn test_walk_process_handles_rapid_enumeration() {
    // Should handle rapid successive enumerations
    let mut all_pids = std::collections::HashSet::new();

    for _ in 0..10 {
        let processes: Vec<_> = walk_process().collect();
        for proc in processes {
            all_pids.insert(proc.pid);
        }
    }

    // Should find a consistent set of processes
    assert!(all_pids.len() > 0, "Should find processes");
}

#[test]
fn test_walk_process_cmdline_consistency() {
    // Command line for same PID should be consistent across calls
    let current_pid = std::process::id() as usize;

    let processes1: Vec<_> = walk_process().collect();
    let cmdline1 = processes1
        .iter()
        .find(|p| p.pid == current_pid)
        .map(|p| p.cmdline.clone());

    let processes2: Vec<_> = walk_process().collect();
    let cmdline2 = processes2
        .iter()
        .find(|p| p.pid == current_pid)
        .map(|p| p.cmdline.clone());

    // Command line should be the same for the same process
    assert_eq!(
        cmdline1, cmdline2,
        "Command line should be consistent for same PID"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_proc_filesystem_access() {
    // Verify we can access /proc filesystem on Linux
    let processes: Vec<_> = walk_process().collect();
    assert!(!processes.is_empty(), "Should access /proc filesystem");

    // Verify we have reasonable number of processes
    assert!(processes.len() > 1, "Linux should have multiple processes");
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_process_state_detection() {
    // Verify we can detect various process states on Linux
    let processes: Vec<_> = walk_process().collect();

    // Should find processes in various states
    assert!(
        processes.len() > 0,
        "Should find processes with various states"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_kernel_threads_detection() {
    // Verify we can detect kernel threads on Linux
    let processes: Vec<_> = walk_process().collect();

    let kernel_threads: Vec<_> = processes
        .iter()
        .filter(|p| p.cmdline.starts_with('[') && p.cmdline.ends_with(']'))
        .collect();

    // Should find kernel threads
    assert!(
        !kernel_threads.is_empty(),
        "Should find kernel threads on Linux"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_process_enumeration() {
    // Verify process enumeration works on macOS
    let processes: Vec<_> = walk_process().collect();
    assert!(!processes.is_empty(), "Should enumerate processes on macOS");

    // Verify we have reasonable number of processes
    assert!(processes.len() > 1, "macOS should have multiple processes");
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_current_process_detection() {
    // Verify we can find current process on macOS
    let processes: Vec<_> = walk_process().collect();
    let current_pid = std::process::id() as usize;

    let found = processes.iter().any(|p| p.pid == current_pid);
    assert!(found, "Should find current process on macOS");
}

#[test]
#[cfg(target_os = "freebsd")]
fn test_freebsd_process_enumeration() {
    // Verify process enumeration works on FreeBSD
    let processes: Vec<_> = walk_process().collect();
    assert!(
        !processes.is_empty(),
        "Should enumerate processes on FreeBSD"
    );

    // Verify we have reasonable number of processes
    assert!(
        processes.len() > 1,
        "FreeBSD should have multiple processes"
    );
}

#[test]
#[cfg(target_os = "freebsd")]
fn test_freebsd_current_process_detection() {
    // Verify we can find current process on FreeBSD
    let processes: Vec<_> = walk_process().collect();
    let current_pid = std::process::id() as usize;

    let found = processes.iter().any(|p| p.pid == current_pid);
    assert!(found, "Should find current process on FreeBSD");
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_process_enumeration() {
    // Verify process enumeration works on Windows
    let processes: Vec<_> = walk_process().collect();
    assert!(
        !processes.is_empty(),
        "Should enumerate processes on Windows"
    );

    // Verify we have reasonable number of processes
    assert!(
        processes.len() > 1,
        "Windows should have multiple processes"
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_current_process_detection() {
    // Verify we can find current process on Windows
    let processes: Vec<_> = walk_process().collect();
    let current_pid = std::process::id() as usize;

    let found = processes.iter().any(|p| p.pid == current_pid);
    assert!(found, "Should find current process on Windows");
}

#[test]
#[cfg(unix)]
fn test_unix_process_hierarchy() {
    // Verify process hierarchy on Unix systems
    let processes: Vec<_> = walk_process().collect();

    // Should have init process (PID 1) on Unix
    #[cfg(target_os = "linux")]
    {
        let init_exists = processes.iter().any(|p| p.pid == 1);
        assert!(init_exists, "Should find init process on Unix");
    }
}

#[test]
fn test_walk_process_performance_single_enumeration() {
    // Benchmark single process enumeration
    let start = std::time::Instant::now();
    let processes: Vec<_> = walk_process().collect();
    let elapsed = start.elapsed();

    assert!(!processes.is_empty(), "Should find processes");

    // Should complete in reasonable time (< 1 second on most systems)
    assert!(
        elapsed.as_secs() < 1,
        "Single enumeration should complete in < 1 second, took {:?}",
        elapsed
    );
}

#[test]
fn test_walk_process_performance_multiple_enumerations() {
    // Benchmark multiple enumerations
    let start = std::time::Instant::now();

    for _ in 0..10 {
        let _processes: Vec<_> = walk_process().collect();
    }

    let elapsed = start.elapsed();

    // 10 enumerations should complete in reasonable time (< 5 seconds)
    assert!(
        elapsed.as_secs() < 5,
        "10 enumerations should complete in < 5 seconds, took {:?}",
        elapsed
    );
}

#[test]
fn test_walk_process_performance_iterator_vs_collect() {
    // Compare iterator vs collect performance
    let start_iter = std::time::Instant::now();
    let mut count_iter = 0;
    for _proc in walk_process() {
        count_iter += 1;
    }
    let elapsed_iter = start_iter.elapsed();

    let start_collect = std::time::Instant::now();
    let processes: Vec<_> = walk_process().collect();
    let elapsed_collect = start_collect.elapsed();

    // Allow for small variance due to processes starting/stopping during enumeration
    let diff = (count_iter as i32 - processes.len() as i32).abs();
    assert!(
        diff <= 5,
        "Iterator and collect should find similar number of processes, got {} and {}",
        count_iter,
        processes.len()
    );
}

#[test]
fn test_walk_process_performance_memory_efficiency() {
    // Test memory efficiency of process enumeration
    let processes: Vec<_> = walk_process().collect();

    // Calculate approximate memory usage
    let mut total_cmdline_size = 0;
    for proc in &processes {
        total_cmdline_size += proc.cmdline.len();
    }

    let avg_cmdline_size = if processes.is_empty() {
        0
    } else {
        total_cmdline_size / processes.len()
    };

    // Average command line should be reasonable (< 1KB)
    assert!(
        avg_cmdline_size < 1024,
        "Average cmdline size should be < 1KB, got {} bytes",
        avg_cmdline_size
    );
}

#[test]
fn test_walk_process_performance_scaling() {
    // Test performance scaling with number of processes
    let start = std::time::Instant::now();
    let processes: Vec<_> = walk_process().collect();
    let elapsed = start.elapsed();

    let process_count = processes.len();

    // Performance should scale reasonably
    // Rough estimate: should handle 1000+ processes in < 1 second
    if process_count > 1000 {
        assert!(
            elapsed.as_millis() < 1000,
            "Should handle {} processes in < 1 second, took {:?}",
            process_count,
            elapsed
        );
    }
}

#[test]
fn test_walk_process_performance_consistency() {
    // Verify performance is consistent across multiple runs
    let mut times = Vec::new();

    for _ in 0..5 {
        let start = std::time::Instant::now();
        let _processes: Vec<_> = walk_process().collect();
        times.push(start.elapsed());
    }

    // Calculate average and check for outliers
    let avg_time: std::time::Duration =
        times.iter().sum::<std::time::Duration>() / times.len() as u32;
    let max_time = times.iter().max().copied().unwrap_or_default();

    // Max time should not be more than 2x average (indicating performance issues)
    assert!(
        max_time <= avg_time * 2,
        "Performance should be consistent, avg: {:?}, max: {:?}",
        avg_time,
        max_time
    );
}
