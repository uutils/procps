fn main() {
    #[cfg(target_os = "linux")]
    pkg_config::find_library("libsystemd").unwrap();
}
