// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use pretty_assertions::assert_eq;
use regex::Regex;

use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

// TODO: make tests combineable (e.g. test --total --human)

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_no_args() {
    let output = new_ucmd!().succeeds().stdout_move_str();

    assert_eq!(output.len(), 207);
    assert_default_format(&output);
}

#[test]
fn test_line() {
    let pattern = Regex::new(r"^SwapUse +\d+ CachUse +\d+ {2}MemUse +\d+ MemFree +\d+\n$").unwrap();

    for arg in ["-L", "--line"] {
        new_ucmd!().arg(arg).succeeds().stdout_matches(&pattern);
    }

    // ensure --line "wins"
    for arg in ["--lohi", "--total", "--committed", "--wide"] {
        new_ucmd!()
            .arg(arg)
            .arg("--line")
            .arg(arg)
            .succeeds()
            .stdout_matches(&pattern);
    }
}

#[test]
fn test_wide() {
    let header_pattern = r"^ {15}total {8}used {8}free {6}shared {5}buffers {7}cache {3}available$";
    let mem_pattern = r"^Mem:( +\d+){7}$";
    let swap_pattern = r"^Swap: ( +\d+){3}$";

    let patterns = vec![
        Regex::new(header_pattern).unwrap(),
        Regex::new(mem_pattern).unwrap(),
        Regex::new(swap_pattern).unwrap(),
    ];

    for arg in ["-w", "--wide"] {
        let binding = new_ucmd!().arg(arg).succeeds();
        let output = binding.stdout_str();

        // The total number of character is always fixed
        assert_eq!(output.len(), 231);

        // Check the format for each line output
        let mut lines = output.lines();
        for pattern in &patterns {
            assert!(pattern.is_match(lines.next().unwrap()));
        }
    }
}

#[test]
fn test_total() {
    for arg in ["-t", "--total"] {
        let result = new_ucmd!().arg(arg).succeeds();
        assert_eq!(result.stdout_str().lines().count(), 4);
        assert!(result
            .stdout_str()
            .lines()
            .last()
            .unwrap()
            .starts_with("Total:"));
    }
}

#[test]
fn test_count() {
    for arg in ["-c", "--count"] {
        let output = new_ucmd!()
            // without -s, there would be a delay of 1s between the output of the
            // two blocks
            .args(&[arg, "2", "-s", "0.00001"])
            .succeeds()
            .stdout_move_str();

        let lines: Vec<&str> = output.lines().collect();

        assert_default_format(&lines[..3].join("\n"));
        assert!(lines[3].is_empty());
        assert_default_format(&lines[4..].join("\n"));
    }
}

#[test]
fn test_count_line() {
    let re = Regex::new(r"^SwapUse +\d+ CachUse +\d+ {2}MemUse +\d+ MemFree +\d+$").unwrap();

    let output = new_ucmd!()
        .args(&["--count", "2", "--line", "-s", "0.00001"])
        .succeeds()
        .stdout_move_str();

    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(2, lines.len());

    assert!(re.is_match(lines[0]));
    assert!(re.is_match(lines[1]));
}

#[test]
fn test_count_zero() {
    new_ucmd!()
        .arg("--count=0")
        .fails()
        .code_is(1)
        .stderr_only("free: count argument must be greater than 0\n");
}

#[test]
fn test_lohi() {
    for arg in ["-l", "--lohi"] {
        let result = new_ucmd!().arg(arg).succeeds();
        assert_eq!(result.stdout_str().lines().count(), 5);
        let lines = result.stdout_str().lines().collect::<Vec<&str>>();
        assert!(lines[2].starts_with("Low:"));
        assert!(lines[3].starts_with("High:"));
    }
}

#[test]
fn test_committed() {
    for arg in ["-v", "--committed"] {
        let result = new_ucmd!().arg(arg).succeeds();
        assert_eq!(result.stdout_str().lines().count(), 4);
        assert!(result
            .stdout_str()
            .lines()
            .last()
            .unwrap()
            .starts_with("Comm:"));
    }
}

#[test]
fn test_seconds_zero() {
    for arg in ["-s", "--seconds"] {
        new_ucmd!()
            .arg(arg)
            .arg("0")
            .fails()
            .code_is(1)
            .stderr_only("free: seconds argument must be greater than 0\n");
    }
}

#[test]
fn test_unit() {
    fn extract_total(re: &Regex, output: &str) -> u64 {
        re.captures(output)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse::<u64>()
            .unwrap()
    }

    let kibi_output = new_ucmd!().succeeds().stdout_move_str();
    let total_mem_re = Regex::new(r"Mem:\s+(\d{1,12})").unwrap();
    let total_swap_re = Regex::new(r"Swap:\s+(\d{1,12})").unwrap();
    let total_mem_bytes = extract_total(&total_mem_re, &kibi_output) * 1024;
    let total_swap_bytes = extract_total(&total_swap_re, &kibi_output) * 1024;

    let base: u64 = 1024;
    let base_si: u64 = 1000;
    for (args, divisor) in vec![
        (vec!["--kilo"], base_si),
        (vec!["--mega"], base_si.pow(2)),
        (vec!["--giga"], base_si.pow(3)),
        (vec!["--tera"], base_si.pow(4)),
        (vec!["--peta"], base_si.pow(5)),
        (vec!["--kilo", "--si"], base_si),
        (vec!["--mega", "--si"], base_si.pow(2)),
        (vec!["--giga", "--si"], base_si.pow(3)),
        (vec!["--tera", "--si"], base_si.pow(4)),
        (vec!["--peta", "--si"], base_si.pow(5)),
        (vec!["--kibi"], base),
        (vec!["--mebi"], base.pow(2)),
        (vec!["--gibi"], base.pow(3)),
        (vec!["--tebi"], base.pow(4)),
        (vec!["--pebi"], base.pow(5)),
        (vec!["--kibi", "--si"], base_si),
        (vec!["--mebi", "--si"], base_si.pow(2)),
        (vec!["--gibi", "--si"], base_si.pow(3)),
        (vec!["--tebi", "--si"], base_si.pow(4)),
        (vec!["--pebi", "--si"], base_si.pow(5)),
        (vec![], base),
        (vec!["--si"], base_si),
    ] {
        let output = new_ucmd!().args(&args).succeeds().stdout_move_str();
        let total_mem = extract_total(&total_mem_re, &output);
        let total_swap = extract_total(&total_swap_re, &output);
        assert_eq!(total_mem, total_mem_bytes / divisor);
        assert_eq!(total_swap, total_swap_bytes / divisor);
    }
}

fn assert_default_format(s: &str) {
    let header_pattern = r"^ {15}total {8}used {8}free {6}shared {2}buff/cache {3}available$";
    let mem_pattern = r"^Mem:( +\d+){6}$";
    let swap_pattern = r"^Swap: ( +\d+){3}$";

    let patterns = vec![
        Regex::new(header_pattern).unwrap(),
        Regex::new(mem_pattern).unwrap(),
        Regex::new(swap_pattern).unwrap(),
    ];

    // Check the format for each line output
    let mut lines = s.lines();
    for pattern in patterns {
        assert!(pattern.is_match(lines.next().unwrap()));
    }
}
