// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::maps_format_parser::{parse_map_line, MapLine};
use std::io::{Error, ErrorKind};

// Represents a parsed single entry from /proc/<PID>/smaps for the extended formats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmapEntry {
    pub map_line: MapLine,
    pub size_in_kb: u64,
    pub kernel_page_size_in_kb: u64,
    pub mmu_page_size_in_kb: u64,
    pub rss_in_kb: u64,
    pub pss_in_kb: u64,
    pub pss_dirty_in_kb: u64,
    pub shared_clean_in_kb: u64,
    pub shared_dirty_in_kb: u64,
    pub private_clean_in_kb: u64,
    pub private_dirty_in_kb: u64,
    pub referenced_in_kb: u64,
    pub anonymous_in_kb: u64,
    pub ksm_in_kb: u64,
    pub lazy_free_in_kb: u64,
    pub anon_huge_pages_in_kb: u64,
    pub shmem_pmd_mapped_in_kb: u64,
    pub file_pmd_mapped_in_kb: u64,
    pub shared_hugetlb_in_kb: u64,
    pub private_hugetlb_in_kb: u64,
    pub swap_in_kb: u64,
    pub swap_pss_in_kb: u64,
    pub locked_in_kb: u64,
    pub thp_eligible: u64,
    pub vmflags: String,
}

// Parses entries from /proc/<PID>/smaps. See
// https://www.kernel.org/doc/html/latest/filesystems/proc.html for details about the expected
// format.
//
// # Errors
//
// Will return an `Error` if the format is incorrect.
pub fn parse_smap_entries(contents: &str) -> Result<Vec<SmapEntry>, Error> {
    let mut smap_entries = Vec::new();
    let mut smap_entry = SmapEntry::default();
    let mut is_entry_modified = false;

    for line in contents.lines() {
        let map_line = parse_map_line(line);
        if let Ok(map_line) = map_line {
            smap_entry.map_line = map_line;
            is_entry_modified = true;
        } else {
            let (key, val) = line
                .split_once(':')
                .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
            let val = val.trim();

            match key {
                "VmFlags" => {
                    smap_entry.vmflags = val.into();
                    smap_entries.push(smap_entry.clone());
                    smap_entry = SmapEntry::default();
                    is_entry_modified = false;
                }
                "THPeligible" => smap_entry.thp_eligible = get_smap_item_value(val)?,
                _ => {
                    let val = val
                        .strip_suffix(" kB")
                        .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
                    let val = get_smap_item_value(val)?;
                    match key {
                        "Size" => smap_entry.size_in_kb = val,
                        "KernelPageSize" => smap_entry.kernel_page_size_in_kb = val,
                        "MMUPageSize" => smap_entry.mmu_page_size_in_kb = val,
                        "Rss" => smap_entry.rss_in_kb = val,
                        "Pss" => smap_entry.pss_in_kb = val,
                        "Pss_Dirty" => smap_entry.pss_dirty_in_kb = val,
                        "Shared_Clean" => smap_entry.shared_clean_in_kb = val,
                        "Shared_Dirty" => smap_entry.shared_dirty_in_kb = val,
                        "Private_Clean" => smap_entry.private_clean_in_kb = val,
                        "Private_Dirty" => smap_entry.private_dirty_in_kb = val,
                        "Referenced" => smap_entry.referenced_in_kb = val,
                        "Anonymous" => smap_entry.anonymous_in_kb = val,
                        "KSM" => smap_entry.ksm_in_kb = val,
                        "LazyFree" => smap_entry.lazy_free_in_kb = val,
                        "AnonHugePages" => smap_entry.anon_huge_pages_in_kb = val,
                        "ShmemPmdMapped" => smap_entry.shmem_pmd_mapped_in_kb = val,
                        "FilePmdMapped" => smap_entry.file_pmd_mapped_in_kb = val,
                        "Shared_Hugetlb" => smap_entry.shared_hugetlb_in_kb = val,
                        "Private_Hugetlb" => smap_entry.private_hugetlb_in_kb = val,
                        "Swap" => smap_entry.swap_in_kb = val,
                        "SwapPss" => smap_entry.swap_pss_in_kb = val,
                        "Locked" => smap_entry.locked_in_kb = val,
                        _ => (),
                    }
                }
            }
        }
    }

    if is_entry_modified {
        smap_entries.push(smap_entry);
    }

    Ok(smap_entries)
}

fn get_smap_item_value(val: &str) -> Result<u64, Error> {
    val.parse::<u64>()
        .map_err(|_| Error::from(ErrorKind::InvalidData))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::maps_format_parser::Perms;

    fn create_smap_entry(
        address: &str,
        perms: Perms,
        offset: &str,
        device: &str,
        inode: u64,
        mapping: &str,
        size_in_kb: u64,
        kernel_page_size_in_kb: u64,
        mmu_page_size_in_kb: u64,
        rss_in_kb: u64,
        pss_in_kb: u64,
        pss_dirty_in_kb: u64,
        shared_clean_in_kb: u64,
        shared_dirty_in_kb: u64,
        private_clean_in_kb: u64,
        private_dirty_in_kb: u64,
        referenced_in_kb: u64,
        anonymous_in_kb: u64,
        ksm_in_kb: u64,
        lazy_free_in_kb: u64,
        anon_huge_pages_in_kb: u64,
        shmem_pmd_mapped_in_kb: u64,
        file_pmd_mapped_in_kb: u64,
        shared_hugetlb_in_kb: u64,
        private_hugetlb_in_kb: u64,
        swap_in_kb: u64,
        swap_pss_in_kb: u64,
        locked_in_kb: u64,
        thp_eligible: u64,
        vmflags: &str,
    ) -> SmapEntry {
        SmapEntry {
            map_line: MapLine {
                address: address.to_string(),
                size_in_kb,
                perms,
                offset: offset.to_string(),
                device: device.to_string(),
                inode,
                mapping: mapping.to_string(),
            },
            size_in_kb,
            kernel_page_size_in_kb,
            mmu_page_size_in_kb,
            rss_in_kb,
            pss_in_kb,
            pss_dirty_in_kb,
            shared_clean_in_kb,
            shared_dirty_in_kb,
            private_clean_in_kb,
            private_dirty_in_kb,
            referenced_in_kb,
            anonymous_in_kb,
            ksm_in_kb,
            lazy_free_in_kb,
            anon_huge_pages_in_kb,
            shmem_pmd_mapped_in_kb,
            file_pmd_mapped_in_kb,
            shared_hugetlb_in_kb,
            private_hugetlb_in_kb,
            swap_in_kb,
            swap_pss_in_kb,
            locked_in_kb,
            thp_eligible,
            vmflags: vmflags.to_string(),
        }
    }

    #[test]
    fn test_parse_smap_entries() {
        let data = [
            (
                vec![create_smap_entry(
                    "0000560880413000", Perms::from("r--p"), "0000000000000000", "008:00008", 10813151, "konsole",
                    180, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, "rd mr mw me dw sd")],
                concat!(
                    "560880413000-560880440000 r--p 00000000 08:08 10813151                   /usr/bin/konsole\n",
                    "Size:                180 kB\n",
                    "KernelPageSize:        1 kB\n",
                    "MMUPageSize:           2 kB\n",
                    "Rss:                   3 kB\n",
                    "Pss:                   4 kB\n",
                    "Pss_Dirty:             5 kB\n",
                    "Shared_Clean:          6 kB\n",
                    "Shared_Dirty:          7 kB\n",
                    "Private_Clean:         8 kB\n",
                    "Private_Dirty:         9 kB\n",
                    "Referenced:           10 kB\n",
                    "Anonymous:            11 kB\n",
                    "KSM:                  12 kB\n",
                    "LazyFree:             13 kB\n",
                    "AnonHugePages:        14 kB\n",
                    "ShmemPmdMapped:       15 kB\n",
                    "FilePmdMapped:        16 kB\n",
                    "Shared_Hugetlb:       17 kB\n",
                    "Private_Hugetlb:      18 kB\n",
                    "Swap:                 19 kB\n",
                    "SwapPss:              20 kB\n",
                    "Locked:               21 kB\n",
                    "THPeligible:           22\n",
                    "VmFlags: rd mr mw me dw sd \n")
            ),
            (
                vec![create_smap_entry(
                    "000071af50000000", Perms::from("rw-p"), "0000000000000000", "000:00000", 0, "  [ anon ]",
                    132, 4, 4, 128, 9, 9, 128, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd mr mw me sd")],
                concat!(
                    "71af50000000-71af50021000 rw-p 00000000 00:00 0 \n",
                    "Size:                132 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                 128 kB\n",
                    "Pss:                   9 kB\n",
                    "Pss_Dirty:             9 kB\n",
                    "Shared_Clean:        128 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:         0 kB\n",
                    "Referenced:          128 kB\n",
                    "Anonymous:             0 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd mr mw me sd \n")
            ),
            (
                vec![create_smap_entry(
                    "00007ffc3f8df000", Perms::from("rw-p"), "0000000000000000", "000:00000", 0, "  [ stack ]",
                    132, 4, 4, 108, 108, 108, 0, 0, 0, 108, 108, 108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd wr mr mw me gd ac")],
                concat!(
                    "7ffc3f8df000-7ffc3f900000 rw-p 00000000 00:00 0                          [stack]\n",
                    "Size:                132 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                 108 kB\n",
                    "Pss:                 108 kB\n",
                    "Pss_Dirty:           108 kB\n",
                    "Shared_Clean:          0 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:       108 kB\n",
                    "Referenced:          108 kB\n",
                    "Anonymous:           108 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd wr mr mw me gd ac\n")
            ),
            (
                vec![create_smap_entry(
                    "000071af8c9e6000", Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "  [ anon ]",
                    16, 4, 4, 16, 16, 16, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd wr mr mw me ac sd")],
                concat!(
                    "71af8c9e6000-71af8c9ea000 rw-s 105830000 00:10 1075                      anon_inode:i915.gem\n",
                    "Size:                 16 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                  16 kB\n",
                    "Pss:                  16 kB\n",
                    "Pss_Dirty:            16 kB\n",
                    "Shared_Clean:          0 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:        16 kB\n",
                    "Referenced:           16 kB\n",
                    "Anonymous:            16 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd wr mr mw me ac sd\n")
            ),
            (
                vec![create_smap_entry(
                    "000071af6cf0c000", Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "memfd:wayland-shm (deleted)",
                    3560, 4, 4, 532, 108, 0, 524, 0, 8, 0, 532, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd mr mw me sd")],
                concat!(
                    "71af6cf0c000-71af6d286000 rw-s 00000000 00:01 256481                     /memfd:wayland-shm (deleted)\n",
                    "Size:               3560 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                 532 kB\n",
                    "Pss:                 108 kB\n",
                    "Pss_Dirty:             0 kB\n",
                    "Shared_Clean:        524 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         8 kB\n",
                    "Private_Dirty:         0 kB\n",
                    "Referenced:          532 kB\n",
                    "Anonymous:             0 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd mr mw me sd \n")
            ),
            (
                vec![create_smap_entry(
                    "ffffffffff600000", Perms::from("--xp"), "0000000000000000", "000:00000", 0, "  [ anon ]",
                    4, 4, 4, 4, 4, 4, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd wr mr mw me ac sd")],
                concat!(
                    "ffffffffff600000-ffffffffff601000 --xp 00000000 00:00 0                  [vsyscall]\n",
                    "Size:                  4 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                   4 kB\n",
                    "Pss:                   4 kB\n",
                    "Pss_Dirty:             4 kB\n",
                    "Shared_Clean:          0 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:         4 kB\n",
                    "Referenced:            4 kB\n",
                    "Anonymous:             4 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd wr mr mw me ac sd\n")
            ),
            (
                vec![create_smap_entry(
                    "00005e8187da8000", Perms::from("r--p"), "0000000000000000", "008:00008", 9524160, "hello   world",
                    24, 4, 4, 24, 0, 0, 24, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, "rd ex mr mw me sd")],
                concat!(
                    "5e8187da8000-5e8187dae000 r--p 00000000 08:08 9524160                    /usr/bin/hello   world\n",
                    "Size:                 24 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                  24 kB\n",
                    "Pss:                   0 kB\n",
                    "Pss_Dirty:             0 kB\n",
                    "Shared_Clean:         24 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:         0 kB\n",
                    "Referenced:           24 kB\n",
                    "Anonymous:             0 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd ex mr mw me sd\n")
            ),
            (
                vec![
                    create_smap_entry(
                        "000071af8c9e6000", Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "  [ anon ]",
                        16, 4, 4, 16, 16, 16, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, "rd wr mr mw me ac sd"),
                    create_smap_entry(
                        "000071af6cf0c000", Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "memfd:wayland-shm (deleted)",
                        3560, 4, 4, 532, 108, 0, 524, 0, 8, 0, 532, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, "rd mr mw me sd"),
                ],
                concat!(
                    "71af8c9e6000-71af8c9ea000 rw-s 105830000 00:10 1075                      anon_inode:i915.gem\n",
                    "Size:                 16 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                  16 kB\n",
                    "Pss:                  16 kB\n",
                    "Pss_Dirty:            16 kB\n",
                    "Shared_Clean:          0 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         0 kB\n",
                    "Private_Dirty:        16 kB\n",
                    "Referenced:           16 kB\n",
                    "Anonymous:            16 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd wr mr mw me ac sd\n",
                    "71af6cf0c000-71af6d286000 rw-s 00000000 00:01 256481                     /memfd:wayland-shm (deleted)\n",
                    "Size:               3560 kB\n",
                    "KernelPageSize:        4 kB\n",
                    "MMUPageSize:           4 kB\n",
                    "Rss:                 532 kB\n",
                    "Pss:                 108 kB\n",
                    "Pss_Dirty:             0 kB\n",
                    "Shared_Clean:        524 kB\n",
                    "Shared_Dirty:          0 kB\n",
                    "Private_Clean:         8 kB\n",
                    "Private_Dirty:         0 kB\n",
                    "Referenced:          532 kB\n",
                    "Anonymous:             0 kB\n",
                    "KSM:                   0 kB\n",
                    "LazyFree:              0 kB\n",
                    "AnonHugePages:         0 kB\n",
                    "ShmemPmdMapped:        0 kB\n",
                    "FilePmdMapped:         0 kB\n",
                    "Shared_Hugetlb:        0 kB\n",
                    "Private_Hugetlb:       0 kB\n",
                    "Swap:                  0 kB\n",
                    "SwapPss:               0 kB\n",
                    "Locked:                0 kB\n",
                    "THPeligible:            0\n",
                    "VmFlags: rd mr mw me sd \n")
            ),
        ];

        for (expected_smap_entries, entries) in data {
            assert_eq!(expected_smap_entries, parse_smap_entries(entries).unwrap());
        }
    }
}
