// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[cfg(target_os = "linux")]
mod linux {

    use uutests::new_ucmd;

    #[test]
    fn test_get_simple() {
        new_ucmd!()
            .arg("kernel.ostype")
            .arg("fs.overflowuid")
            .succeeds()
            .stdout_is("kernel.ostype = Linux\nfs.overflowuid = 65534\n");
    }

    #[test]
    fn test_get_value_only() {
        new_ucmd!()
            .arg("-n")
            .arg("kernel.ostype")
            .arg("fs.overflowuid")
            .succeeds()
            .stdout_is("Linux\n65534\n");
    }

    #[test]
    fn test_get_key_only() {
        new_ucmd!()
            .arg("-N")
            .arg("kernel.ostype")
            .arg("fs.overflowuid")
            .succeeds()
            .stdout_is("kernel.ostype\nfs.overflowuid\n");
    }

    #[test]
    fn test_continues_on_error() {
        new_ucmd!()
            .arg("nonexisting")
            .arg("kernel.ostype")
            .fails()
            .stdout_is("kernel.ostype = Linux\n")
            .stderr_is("sysctl: error reading key 'nonexisting': No such file or directory\n");
    }

    #[test]
    fn test_names_with_quiet() {
        new_ucmd!()
            .args(&["-N", "-q", "kernel.ostype"])
            .fails()
            .code_is(1)
            .no_stdout()
            .stderr_is("sysctl: options -N and -q cannot coexist\nTry `sysctl --help' for more information.\n");
        // With -a, -N and -q together still list every name, as procps-ng does.
        new_ucmd!()
            .args(&["-a", "-N", "-q"])
            .run()
            .stdout_contains_line("kernel.ostype")
            .stderr_does_not_contain("cannot coexist");
    }

    #[test]
    fn test_ignoring_errors() {
        new_ucmd!()
            .arg("-e")
            .arg("nonexisting")
            .arg("nonexisting2=foo")
            .arg("kernel.ostype")
            .succeeds()
            .stdout_is("kernel.ostype = Linux\n")
            .stderr_is("");
    }

    #[test]
    fn test_get_directory() {
        let poolsize = std::fs::read_to_string("/proc/sys/kernel/random/poolsize").unwrap();
        let poolsize = poolsize.trim_end();
        new_ucmd!()
            .arg("kernel.random")
            .succeeds()
            .stdout_contains_line(format!("kernel.random.poolsize = {poolsize}"))
            .stdout_contains("kernel.random.boot_id = ")
            .no_stderr();
        new_ucmd!()
            .args(&["-n", "kernel/random"])
            .succeeds()
            .stdout_contains_line(poolsize)
            .stdout_does_not_contain("kernel.random")
            .no_stderr();
    }

    #[test]
    fn test_get_directory_names() {
        for options in [vec!["-N"], vec!["--names"], vec!["-n", "-N"]] {
            new_ucmd!()
                .args(&options)
                .arg("kernel")
                .succeeds()
                .stdout_contains_line("kernel.ostype")
                .stdout_contains_line("kernel.random.boot_id")
                .stdout_does_not_contain(" = ")
                .no_stderr();
        }
        new_ucmd!()
            .args(&["-N", "/proc/sys/kernel"])
            .fails()
            .code_is(1)
            .no_stdout();
    }

    // procps-ng treats -o and -x as no-op flags (BSD compatibility), not options
    // that take a value. Without SetTrue, clap would consume the next operand.
    #[test]
    fn test_noop_flags_do_not_take_a_value() {
        new_ucmd!()
            .args(&["-o", "kernel.ostype"])
            .succeeds()
            .stdout_is("kernel.ostype = Linux\n");
        new_ucmd!()
            .args(&["-x", "kernel.ostype"])
            .succeeds()
            .stdout_is("kernel.ostype = Linux\n");
        new_ucmd!()
            .args(&["-o", "-n", "kernel.ostype"])
            .succeeds()
            .stdout_is("Linux\n");
        new_ucmd!()
            .args(&["-ox", "kernel.ostype"])
            .succeeds()
            .stdout_is("kernel.ostype = Linux\n");
    }
}

#[cfg(not(target_os = "linux"))]
mod non_linux {

    use uutests::new_ucmd;

    #[test]
    fn test_fails_on_unsupported_platforms() {
        new_ucmd!()
            .arg("-a")
            .fails()
            .code_is(1)
            .stderr_is("sysctl: `sysctl` currently only supports Linux.\n");
    }
}
