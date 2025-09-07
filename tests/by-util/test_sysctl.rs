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
