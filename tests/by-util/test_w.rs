// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help() {
    new_ucmd!()
        .arg("--help")
        .succeeds()
        .stdout_contains("Usage")
        .stdout_contains("Options");
}

#[test]
fn test_no_header() {
    for arg in ["-h", "--no-header"] {
        let cmd = new_ucmd!().arg(arg).succeeds();

        let result = cmd.stdout_str();

        assert!(!result.contains("USER     TTY      LOGIN@   IDLE   JCPU   PCPU WHAT"));
    }
}

#[test]
// As of now, --short is only implemented for Linux
#[cfg(target_os = "linux")]
fn test_option_short() {
    use std::io::IsTerminal;

    use regex::Regex;
    let cmd = new_ucmd!().arg("--short").succeeds();

    let cmd_output = cmd.stdout_str();
    let cmd_output_lines: Vec<&str> = cmd_output.split('\n').collect();
    let line_output_header = cmd_output_lines[0];
    let line_output_data_words: Vec<&str> = cmd_output_lines[1]
        .trim()
        .split(' ')
        .filter(|it| !it.is_empty())
        .collect();

    assert!(line_output_header.contains("USER     TTY      IDLE   WHAT"));
    assert!(!line_output_header.contains("USER     TTY      LOGIN@   IDLE   JCPU   PCPU WHAT"));

    if std::io::stdout().is_terminal() {
        let pattern: Vec<Regex> = vec![
            Regex::new(r"^(\S+)").unwrap(), // USER
            Regex::new(r"(\S+)").unwrap(),  // TERMINAL
            // Regex::new(r"(^$)").unwrap(), // IDLE_TIME => empty str until IDLE_TIME implemented
            Regex::new(r"(\d+\.\d+s)?").unwrap(), // COMMAND
        ];

        assert!(pattern[0].is_match(line_output_data_words[0]));
        assert!(pattern[1].is_match(line_output_data_words[1]));
        // assert!(pattern[2].is_match(line_output_data_words[2]));
        assert!(pattern[2].is_match(line_output_data_words[3]));
    }
}

#[test]
// As of now, output is only implemented for Linux
#[cfg(target_os = "linux")]
fn test_output_format() {
    // Use no header to simplify testing
    let cmd = new_ucmd!().arg("--no-header").succeeds();
    let output_lines = cmd.stdout_str().lines();

    for line in output_lines {
        let line_vec: Vec<String> = line.split_whitespace().map(String::from).collect();
        // Check the time formatting, this should be the third entry in list
        // For now, we are just going to check that that length of time is 5 and it has a colon, else
        // it is possible that a time can look like Fri13, so it can start with a letter and end
        // with a number
        assert!(
            (line_vec[2].contains(':') && line_vec[2].chars().count() == 5)
                || (line_vec[2].starts_with(char::is_alphabetic)
                    && line_vec[2].ends_with(char::is_numeric)
                    && line_vec[2].chars().count() == 5)
        );
        // Assert that there is something in the JCPU and PCPU slots,
        // this will need to be changed when IDLE is implemented
        assert!(!line_vec[3].is_empty() && !line_vec[4].is_empty());
    }
}
