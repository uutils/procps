// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use regex::Regex;
#[cfg(target_os = "linux")]
use std::process;
use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

const NON_EXISTING_PID: &str = "999999";
#[cfg(target_os = "linux")]
const INIT_PID: &str = "1";

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_existing_pid() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    assert_format(pid, &result, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_multiple_existing_pids() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(pid.to_string())
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    let result: Vec<_> = result.lines().collect();

    let re = Regex::new(r"^[1-9]\d*:").unwrap();
    let pos_second_pid = result.iter().rposition(|line| re.is_match(line)).unwrap();
    let (left, right) = result.split_at(pos_second_pid);

    assert_format(pid, &left.join("\n"), false);
    assert_format(pid, &right.join("\n"), false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_non_existing_and_existing_pid() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(NON_EXISTING_PID)
        .arg(pid.to_string())
        .fails();
    let result = result.code_is(42).no_stderr().stdout_str();

    assert_format(pid, result, false);
}

#[test]
fn test_non_existing_pid() {
    new_ucmd!()
        .arg(NON_EXISTING_PID)
        .fails()
        .code_is(42)
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_permission_denied() {
    let result = new_ucmd!()
        .arg(INIT_PID)
        .fails()
        .code_is(1)
        .no_stderr()
        .clone()
        .stdout_move_str();

    assert_cmdline_only(INIT_PID, &result);
}

#[test]
#[cfg(target_os = "linux")]
fn test_extended() {
    let pid = process::id();

    for arg in ["-x", "--extended"] {
        let result = new_ucmd!()
            .arg(arg)
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_extended_format(pid, &result, false);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_more_extended() {
    let pid = process::id();

    let arg = "-X";
    let result = new_ucmd!()
        .arg(arg)
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    assert_more_extended_format(pid, &result, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_most_extended() {
    let pid = process::id();

    let arg = "--XX";
    let result = new_ucmd!()
        .arg(arg)
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    assert_most_extended_format(pid, &result, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_extended_permission_denied() {
    for arg in ["-x", "--extended"] {
        let result = new_ucmd!()
            .arg(arg)
            .arg(INIT_PID)
            .fails()
            .code_is(1)
            .no_stderr()
            .clone()
            .stdout_move_str();

        assert_cmdline_only(INIT_PID, &result);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_device() {
    let pid = process::id();

    for arg in ["-d", "--device"] {
        let result = new_ucmd!()
            .arg(arg)
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_device_format(pid, &result, false);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_device_permission_denied() {
    for arg in ["-d", "--device"] {
        let result = new_ucmd!()
            .arg(arg)
            .arg(INIT_PID)
            .fails()
            .code_is(1)
            .no_stderr()
            .clone()
            .stdout_move_str();

        assert_cmdline_only(INIT_PID, &result);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_showpath() {
    let pid = process::id();

    for arg in ["-p", "--show-path"] {
        // default format
        let result = new_ucmd!()
            .arg(arg)
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_format(pid, &result, true);

        // extended format
        let result = new_ucmd!()
            .arg(arg)
            .arg("--extended")
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_extended_format(pid, &result, true);

        // more-extended format
        let result = new_ucmd!()
            .arg(arg)
            .arg("-X")
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_more_extended_format(pid, &result, true);

        // most-extended format
        let result = new_ucmd!()
            .arg(arg)
            .arg("--XX")
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_most_extended_format(pid, &result, true);

        // device format
        let result = new_ucmd!()
            .arg(arg)
            .arg("--device")
            .arg(pid.to_string())
            .succeeds()
            .stdout_move_str();

        assert_device_format(pid, &result, true);
    }
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

// Ensure `s` has the following format:
//
// 1234:   /some/path
#[cfg(target_os = "linux")]
fn assert_cmdline_only(pid: &str, s: &str) {
    let s = s.trim_end();
    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(s));
}

// Ensure `s` has the following format:
//
// 1234:   /some/path
// 00007ff01c6aa000      8K rw--- ld-linux-x86-64.so.2
// 00007fffa80a6000    132K rw---   [ stack ]
// ffffffffff600000      4K --x--   [ anon ]
// ...
//  total          1040320K
#[cfg(target_os = "linux")]
fn assert_format(pid: u32, s: &str, show_path: bool) {
    let (first_line, rest) = s.split_once('\n').unwrap();
    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(first_line));

    let rest = rest.trim_end();
    let (memory_map, last_line) = rest.rsplit_once('\n').unwrap();
    let re = if show_path {
        Regex::new(r"(?m)^[0-9a-f]{16} +[1-9][0-9]*K (-|r)(-|w)(-|x)(-|s)- (  \[ (anon|stack) \]|/[/a-zA-Z0-9._-]+)$").unwrap()
    } else {
        Regex::new(r"(?m)^[0-9a-f]{16} +[1-9][0-9]*K (-|r)(-|w)(-|x)(-|s)- (  \[ (anon|stack) \]|[a-zA-Z0-9._-]+)$").unwrap()
    };
    assert!(re.is_match(memory_map));

    let re = Regex::new("^ total +[1-9][0-9]*K$").unwrap();
    assert!(re.is_match(last_line));
}

// Ensure `s` has the following extended format (--extended):
//
// 1234:   /some/path
// Address           Kbytes     RSS   Dirty Mode  Mapping
// 000073eb5f4c7000       8       4       0 rw--- ld-linux-x86-64.so.2
// 00007ffd588fc000     132       3      13 rw---   [ stack ]
// ffffffffff600000       4       0       1 --x--   [ anon ]
// ...
// ---------------- ------- ------- ------- (one intentional trailing space)
// total kB             144       7      14
#[cfg(target_os = "linux")]
fn assert_extended_format(pid: u32, s: &str, show_path: bool) {
    let lines: Vec<_> = s.lines().collect();
    let line_count = lines.len();

    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(lines[0]), "failing line: '{}'", lines[0]);

    let expected_header = "Address           Kbytes     RSS   Dirty Mode  Mapping";
    assert_eq!(expected_header, lines[1], "failing line: '{}'", lines[1]);

    let re = if show_path {
        Regex::new(r"^[0-9a-f]{16} +[1-9][0-9]* +\d+ +\d+ (-|r)(-|w)(-|x)(-|s)- (  \[ (anon|stack) \]|/[/a-zA-Z0-9._-]+)$").unwrap()
    } else {
        Regex::new(r"^[0-9a-f]{16} +[1-9][0-9]* +\d+ +\d+ (-|r)(-|w)(-|x)(-|s)- (  \[ (anon|stack) \]|[a-zA-Z0-9._-]+)$").unwrap()
    };

    for line in lines.iter().take(line_count - 2).skip(2) {
        assert!(re.is_match(line), "failing line: '{line}'");
    }

    let expected_separator = "---------------- ------- ------- ------- ";
    assert_eq!(
        expected_separator,
        lines[line_count - 2],
        "failing line: '{}'",
        lines[line_count - 2]
    );

    let re = Regex::new(r"^total kB +[1-9][0-9]* +\d+ +\d+$").unwrap();
    assert!(
        re.is_match(lines[line_count - 1]),
        "failing line: '{}'",
        lines[line_count - 1]
    );
}

// Ensure `s` has the following more extended format (-X):
//
// 1234:   /some/path
//          Address Perm           Offset    Device      Inode Size  Rss  Pss Pss_Dirty Referenced Anonymous LazyFree ShmemPmdMapped FilePmdMapped Shared_Hugetlb Private_Hugetlb Swap SwapPss Locked THPeligible Mapping
// 000073eb5f4c7000 r-xp 0000000000036000 008:00008    2274176 1284 1148 1148         0       1148         0        0              0             0              0               0    0       0      0           0 ld-linux-x86-64.so.2
// 00007ffd588fc000 r--p 0000000000000000 000:00000    2274176   20   20   20        20         20        20        0              0             0              0               0    0       0      0           0 [stack]
// ffffffffff600000 rw-p 0000000000000000 000:00000    2274176   36   36   36        36         36        36        0              0             0              0               0    0       0      0           0 (one intentional trailing space)
// ...
//                                                             ==== ==== ==== ========= ========== ========= ======== ============== ============= ============== =============== ==== ======= ====== =========== (one intentional trailing space)
//                                                             4164 3448 2826       552       3448       552        0              0             0              0               0    0       0      0           0 KB (one intentional trailing space)
#[cfg(target_os = "linux")]
fn assert_more_extended_format(pid: u32, s: &str, show_path: bool) {
    let lines: Vec<_> = s.lines().collect();
    let line_count = lines.len();

    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(lines[0]));

    let re = Regex::new(r"^         Address Perm           Offset    Device      Inode +Size +Rss +Pss +Pss_Dirty +Referenced +Anonymous( +KSM)? +LazyFree +ShmemPmdMapped +FilePmdMapped +Shared_Hugetlb +Private_Hugetlb +Swap +SwapPss +Locked +THPeligible( +ProtectionKey)? +Mapping$").unwrap();
    assert!(re.is_match(lines[1]), "failing line: '{}'", lines[1]);

    let re = if show_path {
        Regex::new(r"^[0-9a-f]{16} (-|r)(-|w)(-|x)(p|s) [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} +\d+ +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? (|\[[a-zA-Z_ ]+\]|/[/a-zA-Z0-9._-]+)$").unwrap()
    } else {
        Regex::new(r"^[0-9a-f]{16} (-|r)(-|w)(-|x)(p|s) [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} +\d+ +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? (|\[[a-zA-Z_ ]+\]|[a-zA-Z0-9._-]+)$").unwrap()
    };

    for line in lines.iter().take(line_count - 2).skip(2) {
        assert!(re.is_match(line), "failing line: '{line}'");
    }

    let re = Regex::new(r"^                                                            +=+ =+ =+ =+ =+ =+( =+)? =+ =+ =+ =+ =+ =+ =+ =+ =+( =+)? $").unwrap();
    assert!(
        re.is_match(lines[line_count - 2]),
        "failing line: '{}'",
        lines[line_count - 2]
    );

    let re = Regex::new(r"^                                                            +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? KB $").unwrap();
    assert!(
        re.is_match(lines[line_count - 1]),
        "failing line: '{}'",
        lines[line_count - 1]
    );
}

// Ensure `s` has the following most extended format (--XX):
//
// 1234:   /some/path
//          Address Perm           Offset    Device      Inode Size KernelPageSize MMUPageSize  Rss  Pss Pss_Dirty Shared_Clean Shared_Dirty Private_Clean Private_Dirty Referenced Anonymous LazyFree AnonHugePages ShmemPmdMapped FilePmdMapped Shared_Hugetlb Private_Hugetlb Swap SwapPss Locked THPeligible              VmFlags Mapping
// 000073eb5f4c7000 r-xp 0000000000036000 008:00008    2274176 1284              4           4 1148 1148         0            0            0          1148             0       1148         0        0             0              0             0              0               0    0       0      0           0       rd ex mr mw me ld-linux-x86-64.so.2
// 00007ffd588fc000 r--p 0000000000000000 000:00000    2274176   20              4           4   20   20        20            0            0             0            20         20        20        0             0              0             0              0               0    0       0      0           0       rd mr mw me ac [stack]
// ffffffffff600000 rw-p 0000000000000000 000:00000    2274176   36              4           4   36   36        36            0            0             0            36         36        36        0             0              0             0              0               0    0       0      0           0    rd wr mr mw me ac (one intentional trailing space)
// ...
//                                                             ==== ============== =========== ==== ==== ========= ============ ============ ============= ============= ========== ========= ======== ============= ============== ============= ============== =============== ==== ======= ====== =========== (one intentional trailing space)
//                                                             4164             92          92 3448 2880       552         1132            0          1764           552       3448       552        0             0              0             0              0               0    0       0      0           0 KB (one intentional trailing space)
#[cfg(target_os = "linux")]
fn assert_most_extended_format(pid: u32, s: &str, show_path: bool) {
    let lines: Vec<_> = s.lines().collect();
    let line_count = lines.len();

    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(lines[0]));

    let re = Regex::new(r"^         Address Perm           Offset    Device      Inode +Size +KernelPageSize +MMUPageSize +Rss +Pss +Pss_Dirty +Shared_Clean +Shared_Dirty +Private_Clean +Private_Dirty +Referenced +Anonymous( +KSM)? +LazyFree +AnonHugePages +ShmemPmdMapped +FilePmdMapped +Shared_Hugetlb +Private_Hugetlb +Swap +SwapPss +Locked +THPeligible( +ProtectionKey)? +VmFlags +Mapping$").unwrap();
    assert!(re.is_match(lines[1]), "failing line: '{}'", lines[1]);

    let re = if show_path {
        Regex::new(r"^[0-9a-f]{16} (-|r)(-|w)(-|x)(p|s) [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} +\d+ +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +([a-z][a-z] )*(|\[[a-zA-Z_ ]+\]|/[/a-zA-Z0-9._-]+)$").unwrap()
    } else {
        Regex::new(r"^[0-9a-f]{16} (-|r)(-|w)(-|x)(p|s) [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} +\d+ +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +([a-z][a-z] )*(|\[[a-zA-Z_ ]+\]|[a-zA-Z0-9._-]+)$").unwrap()
    };

    for line in lines.iter().take(line_count - 2).skip(2) {
        assert!(re.is_match(line), "failing line: '{line}'");
    }

    let re = Regex::new(r"^                                                            +=+ =+ =+ =+ =+ =+ =+ =+ =+ =+ =+ =+( =+)? =+ =+ =+ =+ =+ =+ =+ =+ =+ =+( =+)? $").unwrap();
    assert!(
        re.is_match(lines[line_count - 2]),
        "failing line: '{}'",
        lines[line_count - 2]
    );

    let re = Regex::new(r"^                                                            +[1-9][0-9]* +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+ +\d+( +\d+)? KB $").unwrap();
    assert!(
        re.is_match(lines[line_count - 1]),
        "failing line: '{}'",
        lines[line_count - 1]
    );
}

// Ensure `s` has the following device format (--device):
//
// 1234:   /some/path
// Address           Kbytes Mode  Offset           Device    Mapping
// 000073eb5f4c7000       8 rw--- 0000000000036000 008:00008 ld-linux-x86-64.so.2
// 00007ffd588fc000     132 rw--- 0000000000000000 000:00000   [ stack ]
// ffffffffff600000       4 --x-- 0000000000000000 000:00000   [ anon ]
// ...
// mapped: 3060K    writeable/private: 348K    shared: 0K
#[cfg(target_os = "linux")]
fn assert_device_format(pid: u32, s: &str, show_path: bool) {
    let lines: Vec<_> = s.lines().collect();
    let line_count = lines.len();

    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(lines[0]));

    let expected_header = "Address           Kbytes Mode  Offset           Device    Mapping";
    assert_eq!(expected_header, lines[1]);

    let re = if show_path {
        Regex::new(r"^[0-9a-f]{16} +[1-9][0-9]* (-|r)(-|w)(-|x)(-|s)- [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} (  \[ (anon|stack) \]|/[/a-zA-Z0-9._-]+)$").unwrap()
    } else {
        Regex::new(r"^[0-9a-f]{16} +[1-9][0-9]* (-|r)(-|w)(-|x)(-|s)- [0-9a-f]{16} [0-9a-f]{3}:[0-9a-f]{5} (  \[ (anon|stack) \]|[a-zA-Z0-9._-]+)$").unwrap()
    };

    for line in lines.iter().take(line_count - 1).skip(2) {
        assert!(re.is_match(line), "failing line: {line}");
    }

    let re = Regex::new(r"^mapped: \d+K\s{4}writeable/private: \d+K\s{4}shared: \d+K$").unwrap();
    assert!(
        re.is_match(lines[line_count - 1]),
        "failing line: {}",
        lines[line_count - 1]
    );
}
