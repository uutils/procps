// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::maps_format_parser::{parse_map_line, MapLine};
use crate::pmap_config::pmap_field_name;
use std::io::{Error, ErrorKind};

// Represents a parsed single entry from /proc/<PID>/smaps for the extended formats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmapEntry {
    pub map_line: MapLine,
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
    pub protection_key: u64,
    pub vmflags: String,
}

impl SmapEntry {
    pub fn get_field(&self, field_name: &str) -> String {
        match field_name {
            pmap_field_name::ADDRESS => self.map_line.address.clone(),
            pmap_field_name::PERM => self.map_line.perms.to_string().clone(),
            pmap_field_name::OFFSET => self.map_line.offset.clone(),
            pmap_field_name::DEVICE => self.map_line.device.clone(),
            pmap_field_name::INODE => self.map_line.inode.to_string(),
            pmap_field_name::SIZE => self.map_line.size_in_kb.to_string(),
            pmap_field_name::KERNEL_PAGE_SIZE => self.kernel_page_size_in_kb.to_string(),
            pmap_field_name::MMU_PAGE_SIZE => self.mmu_page_size_in_kb.to_string(),
            pmap_field_name::RSS => self.rss_in_kb.to_string(),
            pmap_field_name::PSS => self.pss_in_kb.to_string(),
            pmap_field_name::PSS_DIRTY => self.pss_dirty_in_kb.to_string(),
            pmap_field_name::SHARED_CLEAN => self.shared_clean_in_kb.to_string(),
            pmap_field_name::SHARED_DIRTY => self.shared_dirty_in_kb.to_string(),
            pmap_field_name::PRIVATE_CLEAN => self.private_clean_in_kb.to_string(),
            pmap_field_name::PRIVATE_DIRTY => self.private_dirty_in_kb.to_string(),
            pmap_field_name::REFERENCED => self.referenced_in_kb.to_string(),
            pmap_field_name::ANONYMOUS => self.anonymous_in_kb.to_string(),
            pmap_field_name::KSM => self.ksm_in_kb.to_string(),
            pmap_field_name::LAZY_FREE => self.lazy_free_in_kb.to_string(),
            pmap_field_name::ANON_HUGE_PAGES => self.anon_huge_pages_in_kb.to_string(),
            pmap_field_name::SHMEM_PMD_MAPPED => self.shmem_pmd_mapped_in_kb.to_string(),
            pmap_field_name::FILE_PMD_MAPPED => self.file_pmd_mapped_in_kb.to_string(),
            pmap_field_name::SHARED_HUGETLB => self.shared_hugetlb_in_kb.to_string(),
            pmap_field_name::PRIVATE_HUGETLB => self.private_hugetlb_in_kb.to_string(),
            pmap_field_name::SWAP => self.swap_in_kb.to_string(),
            pmap_field_name::SWAP_PSS => self.swap_pss_in_kb.to_string(),
            pmap_field_name::LOCKED => self.locked_in_kb.to_string(),
            pmap_field_name::THP_ELIGIBLE => self.thp_eligible.to_string(),
            pmap_field_name::PROTECTION_KEY => self.protection_key.to_string(),
            pmap_field_name::VMFLAGS => self.vmflags.clone(),
            pmap_field_name::MAPPING => self.map_line.mapping.clone(),
            _ => String::new(),
        }
    }
}

// Internal info used to determine the print contents.
#[derive(Debug, Clone, PartialEq)]
pub struct SmapTableInfo {
    pub has_ksm: bool,
    pub has_protection_key: bool,
    // Total value
    pub total_size_in_kb: u64,
    pub total_kernel_page_size_in_kb: u64,
    pub total_mmu_page_size_in_kb: u64,
    pub total_rss_in_kb: u64,
    pub total_pss_in_kb: u64,
    pub total_pss_dirty_in_kb: u64,
    pub total_shared_clean_in_kb: u64,
    pub total_shared_dirty_in_kb: u64,
    pub total_private_clean_in_kb: u64,
    pub total_private_dirty_in_kb: u64,
    pub total_referenced_in_kb: u64,
    pub total_anonymous_in_kb: u64,
    pub total_ksm_in_kb: u64,
    pub total_lazy_free_in_kb: u64,
    pub total_anon_huge_pages_in_kb: u64,
    pub total_shmem_pmd_mapped_in_kb: u64,
    pub total_file_pmd_mapped_in_kb: u64,
    pub total_shared_hugetlb_in_kb: u64,
    pub total_private_hugetlb_in_kb: u64,
    pub total_swap_in_kb: u64,
    pub total_swap_pss_in_kb: u64,
    pub total_locked_in_kb: u64,
    pub total_thp_eligible: u64,
    pub total_protection_key: u64,
    // Width
    pub size_in_kb_width: usize,
    pub kernel_page_size_in_kb_width: usize,
    pub mmu_page_size_in_kb_width: usize,
    pub rss_in_kb_width: usize,
    pub pss_in_kb_width: usize,
    pub pss_dirty_in_kb_width: usize,
    pub shared_clean_in_kb_width: usize,
    pub shared_dirty_in_kb_width: usize,
    pub private_clean_in_kb_width: usize,
    pub private_dirty_in_kb_width: usize,
    pub referenced_in_kb_width: usize,
    pub anonymous_in_kb_width: usize,
    pub ksm_in_kb_width: usize,
    pub lazy_free_in_kb_width: usize,
    pub anon_huge_pages_in_kb_width: usize,
    pub shmem_pmd_mapped_in_kb_width: usize,
    pub file_pmd_mapped_in_kb_width: usize,
    pub shared_hugetlb_in_kb_width: usize,
    pub private_hugetlb_in_kb_width: usize,
    pub swap_in_kb_width: usize,
    pub swap_pss_in_kb_width: usize,
    pub locked_in_kb_width: usize,
    pub thp_eligible_width: usize,
    pub protection_key_width: usize,
    pub vmflags_width: usize,
}

impl Default for SmapTableInfo {
    fn default() -> Self {
        Self {
            has_ksm: false,
            has_protection_key: false,

            total_size_in_kb: 0,
            total_kernel_page_size_in_kb: 0,
            total_mmu_page_size_in_kb: 0,
            total_rss_in_kb: 0,
            total_pss_in_kb: 0,
            total_pss_dirty_in_kb: 0,
            total_shared_clean_in_kb: 0,
            total_shared_dirty_in_kb: 0,
            total_private_clean_in_kb: 0,
            total_private_dirty_in_kb: 0,
            total_referenced_in_kb: 0,
            total_anonymous_in_kb: 0,
            total_ksm_in_kb: 0,
            total_lazy_free_in_kb: 0,
            total_anon_huge_pages_in_kb: 0,
            total_shmem_pmd_mapped_in_kb: 0,
            total_file_pmd_mapped_in_kb: 0,
            total_shared_hugetlb_in_kb: 0,
            total_private_hugetlb_in_kb: 0,
            total_swap_in_kb: 0,
            total_swap_pss_in_kb: 0,
            total_locked_in_kb: 0,
            total_thp_eligible: 0,
            total_protection_key: 0,

            size_in_kb_width: pmap_field_name::SIZE.len(),
            kernel_page_size_in_kb_width: pmap_field_name::KERNEL_PAGE_SIZE.len(),
            mmu_page_size_in_kb_width: pmap_field_name::MMU_PAGE_SIZE.len(),
            rss_in_kb_width: pmap_field_name::RSS.len(),
            pss_in_kb_width: pmap_field_name::PSS.len(),
            pss_dirty_in_kb_width: pmap_field_name::PSS_DIRTY.len(),
            shared_clean_in_kb_width: pmap_field_name::SHARED_CLEAN.len(),
            shared_dirty_in_kb_width: pmap_field_name::SHARED_DIRTY.len(),
            private_clean_in_kb_width: pmap_field_name::PRIVATE_CLEAN.len(),
            private_dirty_in_kb_width: pmap_field_name::PRIVATE_DIRTY.len(),
            referenced_in_kb_width: pmap_field_name::REFERENCED.len(),
            anonymous_in_kb_width: pmap_field_name::ANONYMOUS.len(),
            ksm_in_kb_width: pmap_field_name::KSM.len(),
            lazy_free_in_kb_width: pmap_field_name::LAZY_FREE.len(),
            anon_huge_pages_in_kb_width: pmap_field_name::ANON_HUGE_PAGES.len(),
            shmem_pmd_mapped_in_kb_width: pmap_field_name::SHMEM_PMD_MAPPED.len(),
            file_pmd_mapped_in_kb_width: pmap_field_name::FILE_PMD_MAPPED.len(),
            shared_hugetlb_in_kb_width: pmap_field_name::SHARED_HUGETLB.len(),
            private_hugetlb_in_kb_width: pmap_field_name::PRIVATE_HUGETLB.len(),
            swap_in_kb_width: pmap_field_name::SWAP.len(),
            swap_pss_in_kb_width: pmap_field_name::SWAP_PSS.len(),
            locked_in_kb_width: pmap_field_name::LOCKED.len(),
            thp_eligible_width: pmap_field_name::THP_ELIGIBLE.len(),
            protection_key_width: pmap_field_name::PROTECTION_KEY.len(),
            vmflags_width: pmap_field_name::VMFLAGS.len(),
        }
    }
}

impl SmapTableInfo {
    // Used to determine the field width in custom format.
    pub fn get_width(&self, field_name: &str) -> usize {
        match field_name {
            pmap_field_name::ADDRESS => 16, // See maps_format_parser.rs
            pmap_field_name::PERM => 4,     // See maps_format_parser.rs
            pmap_field_name::OFFSET => 16,  // See maps_format_parser.rs
            pmap_field_name::DEVICE => 9,   // See maps_format_parser.rs
            pmap_field_name::INODE => 10,   // See maps_format_parser.rs
            pmap_field_name::SIZE => self.size_in_kb_width,
            pmap_field_name::KERNEL_PAGE_SIZE => self.kernel_page_size_in_kb_width,
            pmap_field_name::MMU_PAGE_SIZE => self.mmu_page_size_in_kb_width,
            pmap_field_name::RSS => self.rss_in_kb_width,
            pmap_field_name::PSS => self.pss_in_kb_width,
            pmap_field_name::PSS_DIRTY => self.pss_dirty_in_kb_width,
            pmap_field_name::SHARED_CLEAN => self.shared_clean_in_kb_width,
            pmap_field_name::SHARED_DIRTY => self.shared_dirty_in_kb_width,
            pmap_field_name::PRIVATE_CLEAN => self.private_clean_in_kb_width,
            pmap_field_name::PRIVATE_DIRTY => self.private_dirty_in_kb_width,
            pmap_field_name::REFERENCED => self.referenced_in_kb_width,
            pmap_field_name::ANONYMOUS => self.anonymous_in_kb_width,
            pmap_field_name::KSM => self.ksm_in_kb_width,
            pmap_field_name::LAZY_FREE => self.lazy_free_in_kb_width,
            pmap_field_name::ANON_HUGE_PAGES => self.anon_huge_pages_in_kb_width,
            pmap_field_name::SHMEM_PMD_MAPPED => self.shmem_pmd_mapped_in_kb_width,
            pmap_field_name::FILE_PMD_MAPPED => self.file_pmd_mapped_in_kb_width,
            pmap_field_name::SHARED_HUGETLB => self.shared_hugetlb_in_kb_width,
            pmap_field_name::PRIVATE_HUGETLB => self.private_hugetlb_in_kb_width,
            pmap_field_name::SWAP => self.swap_in_kb_width,
            pmap_field_name::SWAP_PSS => self.swap_pss_in_kb_width,
            pmap_field_name::LOCKED => self.locked_in_kb_width,
            pmap_field_name::THP_ELIGIBLE => self.thp_eligible_width,
            pmap_field_name::PROTECTION_KEY => self.protection_key_width,
            pmap_field_name::VMFLAGS => self.vmflags_width,
            _ => 0,
        }
    }

    pub fn get_total(&self, field_name: &str) -> u64 {
        match field_name {
            pmap_field_name::SIZE => self.total_size_in_kb,
            pmap_field_name::KERNEL_PAGE_SIZE => self.total_kernel_page_size_in_kb,
            pmap_field_name::MMU_PAGE_SIZE => self.total_mmu_page_size_in_kb,
            pmap_field_name::RSS => self.total_rss_in_kb,
            pmap_field_name::PSS => self.total_pss_in_kb,
            pmap_field_name::PSS_DIRTY => self.total_pss_dirty_in_kb,
            pmap_field_name::SHARED_CLEAN => self.total_shared_clean_in_kb,
            pmap_field_name::SHARED_DIRTY => self.total_shared_dirty_in_kb,
            pmap_field_name::PRIVATE_CLEAN => self.total_private_clean_in_kb,
            pmap_field_name::PRIVATE_DIRTY => self.total_private_dirty_in_kb,
            pmap_field_name::REFERENCED => self.total_referenced_in_kb,
            pmap_field_name::ANONYMOUS => self.total_anonymous_in_kb,
            pmap_field_name::KSM => self.total_ksm_in_kb,
            pmap_field_name::LAZY_FREE => self.total_lazy_free_in_kb,
            pmap_field_name::ANON_HUGE_PAGES => self.total_anon_huge_pages_in_kb,
            pmap_field_name::SHMEM_PMD_MAPPED => self.total_shmem_pmd_mapped_in_kb,
            pmap_field_name::FILE_PMD_MAPPED => self.total_file_pmd_mapped_in_kb,
            pmap_field_name::SHARED_HUGETLB => self.total_shared_hugetlb_in_kb,
            pmap_field_name::PRIVATE_HUGETLB => self.total_private_hugetlb_in_kb,
            pmap_field_name::SWAP => self.total_swap_in_kb,
            pmap_field_name::SWAP_PSS => self.total_swap_pss_in_kb,
            pmap_field_name::LOCKED => self.total_locked_in_kb,
            pmap_field_name::THP_ELIGIBLE => self.total_thp_eligible,
            pmap_field_name::PROTECTION_KEY => self.total_protection_key,
            _ => 0,
        }
    }
}

// Represents the entire parsed entries from /proc/<PID>/smaps for the extended formats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmapTable {
    pub entries: Vec<SmapEntry>,
    pub info: SmapTableInfo,
}

// Parses entries from /proc/<PID>/smaps. See
// https://www.kernel.org/doc/html/latest/filesystems/proc.html for details about the expected
// format.
//
// # Errors
//
// Will return an `Error` if the format is incorrect.
pub fn parse_smaps(contents: &str) -> Result<SmapTable, Error> {
    let mut smap_table = SmapTable::default();
    let mut smap_entry = SmapEntry::default();

    for (i, line) in contents.lines().enumerate() {
        let map_line = parse_map_line(line);
        if let Ok(map_line) = map_line {
            if i > 0 {
                smap_table.entries.push(smap_entry.clone());
                smap_entry = SmapEntry::default();
            }
            smap_table.info.total_size_in_kb += map_line.size_in_kb;
            smap_entry.map_line = map_line;
        } else {
            let (key, val) = line
                .split_once(':')
                .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
            let val = val.trim();

            if key == pmap_field_name::VMFLAGS {
                smap_entry.vmflags = val.into();
                smap_table.info.vmflags_width =
                    smap_table.info.vmflags_width.max(smap_entry.vmflags.len());
            } else {
                let val = val.strip_suffix(" kB").unwrap_or(val);
                let val = get_smap_item_value(val)?;
                match key {
                    pmap_field_name::SIZE => {
                        if smap_entry.map_line.size_in_kb != val {
                            return Err(Error::from(ErrorKind::InvalidData));
                        }
                    }
                    pmap_field_name::KERNEL_PAGE_SIZE => {
                        smap_entry.kernel_page_size_in_kb = val;
                        smap_table.info.total_kernel_page_size_in_kb += val;
                    }
                    pmap_field_name::MMU_PAGE_SIZE => {
                        smap_entry.mmu_page_size_in_kb = val;
                        smap_table.info.total_mmu_page_size_in_kb += val;
                    }
                    pmap_field_name::RSS => {
                        smap_entry.rss_in_kb = val;
                        smap_table.info.total_rss_in_kb += val;
                    }
                    pmap_field_name::PSS => {
                        smap_entry.pss_in_kb = val;
                        smap_table.info.total_pss_in_kb += val;
                    }
                    pmap_field_name::PSS_DIRTY => {
                        smap_entry.pss_dirty_in_kb = val;
                        smap_table.info.total_pss_dirty_in_kb += val;
                    }
                    pmap_field_name::SHARED_CLEAN => {
                        smap_entry.shared_clean_in_kb = val;
                        smap_table.info.total_shared_clean_in_kb += val;
                    }
                    pmap_field_name::SHARED_DIRTY => {
                        smap_entry.shared_dirty_in_kb = val;
                        smap_table.info.total_shared_dirty_in_kb += val;
                    }
                    pmap_field_name::PRIVATE_CLEAN => {
                        smap_entry.private_clean_in_kb = val;
                        smap_table.info.total_private_clean_in_kb += val;
                    }
                    pmap_field_name::PRIVATE_DIRTY => {
                        smap_entry.private_dirty_in_kb = val;
                        smap_table.info.total_private_dirty_in_kb += val;
                    }
                    pmap_field_name::REFERENCED => {
                        smap_entry.referenced_in_kb = val;
                        smap_table.info.total_referenced_in_kb += val;
                    }
                    pmap_field_name::ANONYMOUS => {
                        smap_entry.anonymous_in_kb = val;
                        smap_table.info.total_anonymous_in_kb += val;
                    }
                    pmap_field_name::KSM => {
                        smap_entry.ksm_in_kb = val;
                        smap_table.info.total_ksm_in_kb += val;
                        smap_table.info.has_ksm = true;
                    }
                    pmap_field_name::LAZY_FREE => {
                        smap_entry.lazy_free_in_kb = val;
                        smap_table.info.total_lazy_free_in_kb += val;
                    }
                    pmap_field_name::ANON_HUGE_PAGES => {
                        smap_entry.anon_huge_pages_in_kb = val;
                        smap_table.info.total_anon_huge_pages_in_kb += val;
                    }
                    pmap_field_name::SHMEM_PMD_MAPPED => {
                        smap_entry.shmem_pmd_mapped_in_kb = val;
                        smap_table.info.total_shmem_pmd_mapped_in_kb += val;
                    }
                    pmap_field_name::FILE_PMD_MAPPED => {
                        smap_entry.file_pmd_mapped_in_kb = val;
                        smap_table.info.total_file_pmd_mapped_in_kb += val;
                    }
                    pmap_field_name::SHARED_HUGETLB => {
                        smap_entry.shared_hugetlb_in_kb = val;
                        smap_table.info.total_shared_hugetlb_in_kb += val;
                    }
                    pmap_field_name::PRIVATE_HUGETLB => {
                        smap_entry.private_hugetlb_in_kb = val;
                        smap_table.info.total_private_hugetlb_in_kb += val;
                    }
                    pmap_field_name::SWAP => {
                        smap_entry.swap_in_kb = val;
                        smap_table.info.total_swap_in_kb += val;
                    }
                    pmap_field_name::SWAP_PSS => {
                        smap_entry.swap_pss_in_kb = val;
                        smap_table.info.total_swap_pss_in_kb += val;
                    }
                    pmap_field_name::LOCKED => {
                        smap_entry.locked_in_kb = val;
                        smap_table.info.total_locked_in_kb += val;
                    }
                    pmap_field_name::THP_ELIGIBLE => {
                        smap_entry.thp_eligible = val;
                        smap_table.info.total_thp_eligible += val;
                    }
                    pmap_field_name::PROTECTION_KEY => {
                        smap_entry.protection_key = val;
                        smap_table.info.total_protection_key += val;
                        smap_table.info.has_protection_key = true;
                    }
                    _ => (),
                }
            }
        }
    }

    if !contents.is_empty() {
        smap_table.entries.push(smap_entry);
    }

    // Update width information
    smap_table.info.size_in_kb_width = smap_table
        .info
        .size_in_kb_width
        .max(smap_table.info.total_size_in_kb.to_string().len());
    smap_table.info.kernel_page_size_in_kb_width =
        smap_table.info.kernel_page_size_in_kb_width.max(
            smap_table
                .info
                .total_kernel_page_size_in_kb
                .to_string()
                .len(),
        );
    smap_table.info.mmu_page_size_in_kb_width = smap_table
        .info
        .mmu_page_size_in_kb_width
        .max(smap_table.info.total_mmu_page_size_in_kb.to_string().len());
    smap_table.info.rss_in_kb_width = smap_table
        .info
        .rss_in_kb_width
        .max(smap_table.info.total_rss_in_kb.to_string().len());
    smap_table.info.pss_in_kb_width = smap_table
        .info
        .pss_in_kb_width
        .max(smap_table.info.total_pss_in_kb.to_string().len());
    smap_table.info.pss_dirty_in_kb_width = smap_table
        .info
        .pss_dirty_in_kb_width
        .max(smap_table.info.total_pss_dirty_in_kb.to_string().len());
    smap_table.info.shared_clean_in_kb_width = smap_table
        .info
        .shared_clean_in_kb_width
        .max(smap_table.info.total_shared_clean_in_kb.to_string().len());
    smap_table.info.shared_dirty_in_kb_width = smap_table
        .info
        .shared_dirty_in_kb_width
        .max(smap_table.info.total_shared_dirty_in_kb.to_string().len());
    smap_table.info.private_clean_in_kb_width = smap_table
        .info
        .private_clean_in_kb_width
        .max(smap_table.info.total_private_clean_in_kb.to_string().len());
    smap_table.info.private_dirty_in_kb_width = smap_table
        .info
        .private_dirty_in_kb_width
        .max(smap_table.info.total_private_dirty_in_kb.to_string().len());
    smap_table.info.referenced_in_kb_width = smap_table
        .info
        .referenced_in_kb_width
        .max(smap_table.info.total_referenced_in_kb.to_string().len());
    smap_table.info.anonymous_in_kb_width = smap_table
        .info
        .anonymous_in_kb_width
        .max(smap_table.info.total_anonymous_in_kb.to_string().len());
    smap_table.info.ksm_in_kb_width = smap_table
        .info
        .ksm_in_kb_width
        .max(smap_table.info.total_ksm_in_kb.to_string().len());
    smap_table.info.lazy_free_in_kb_width = smap_table
        .info
        .lazy_free_in_kb_width
        .max(smap_table.info.total_lazy_free_in_kb.to_string().len());
    smap_table.info.anon_huge_pages_in_kb_width = smap_table.info.anon_huge_pages_in_kb_width.max(
        smap_table
            .info
            .total_anon_huge_pages_in_kb
            .to_string()
            .len(),
    );
    smap_table.info.shmem_pmd_mapped_in_kb_width =
        smap_table.info.shmem_pmd_mapped_in_kb_width.max(
            smap_table
                .info
                .total_shmem_pmd_mapped_in_kb
                .to_string()
                .len(),
        );
    smap_table.info.file_pmd_mapped_in_kb_width = smap_table.info.file_pmd_mapped_in_kb_width.max(
        smap_table
            .info
            .total_file_pmd_mapped_in_kb
            .to_string()
            .len(),
    );
    smap_table.info.shared_hugetlb_in_kb_width = smap_table
        .info
        .shared_hugetlb_in_kb_width
        .max(smap_table.info.total_shared_hugetlb_in_kb.to_string().len());
    smap_table.info.private_hugetlb_in_kb_width = smap_table.info.private_hugetlb_in_kb_width.max(
        smap_table
            .info
            .total_private_hugetlb_in_kb
            .to_string()
            .len(),
    );
    smap_table.info.swap_in_kb_width = smap_table
        .info
        .swap_in_kb_width
        .max(smap_table.info.total_swap_in_kb.to_string().len());
    smap_table.info.swap_pss_in_kb_width = smap_table
        .info
        .swap_pss_in_kb_width
        .max(smap_table.info.total_swap_pss_in_kb.to_string().len());
    smap_table.info.locked_in_kb_width = smap_table
        .info
        .locked_in_kb_width
        .max(smap_table.info.total_locked_in_kb.to_string().len());
    smap_table.info.thp_eligible_width = smap_table
        .info
        .thp_eligible_width
        .max(smap_table.info.total_thp_eligible.to_string().len());
    smap_table.info.protection_key_width = smap_table
        .info
        .protection_key_width
        .max(smap_table.info.total_protection_key.to_string().len());

    Ok(smap_table)
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
        protection_key: u64,
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
            protection_key,
            vmflags: vmflags.to_string(),
        }
    }

    #[test]
    fn test_parse_smaps() {
        let data = [
            (
                vec![create_smap_entry(
                    "0000560880413000", Perms::from("r--p"), "0000000000000000", "008:00008", 10813151, "/usr/bin/konsole",
                    180, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 0, "rd mr mw me dw sd")],
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
                    "SomeUnknownFieldKB:   23 kB\n",
                    "SomeUnknownField:      24\n",
                    "VmFlags: rd mr mw me dw sd \n")
            ),
            (
                vec![create_smap_entry(
                    "000071af50000000", Perms::from("rw-p"), "0000000000000000", "000:00000", 0, "",
                    132, 4, 4, 128, 9, 9, 128, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 2, "rd mr mw me sd")],
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
                    "ProtectionKey:          2\n",
                    "VmFlags: rd mr mw me sd \n")
            ),
            (
                vec![create_smap_entry(
                    "00007ffc3f8df000", Perms::from("rw-p"), "0000000000000000", "000:00000", 0, "[stack]",
                    132, 4, 4, 108, 108, 108, 0, 0, 0, 108, 108, 108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 3, "rd wr mr mw me gd ac")],
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
                    "ProtectionKey:          3\n",
                    "VmFlags: rd wr mr mw me gd ac\n")
            ),
            (
                vec![create_smap_entry(
                    "000071af8c9e6000", Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "anon_inode:i915.gem",
                    16, 4, 4, 16, 16, 16, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, "rd wr mr mw me ac sd")],
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
                    "000071af6cf0c000", Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "/memfd:wayland-shm (deleted)",
                    3560, 4, 4, 532, 108, 0, 524, 0, 8, 0, 532, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, "rd mr mw me sd")],
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
                    "ffffffffff600000", Perms::from("--xp"), "0000000000000000", "000:00000", 0, "[vsyscall]",
                    4, 4, 4, 4, 4, 4, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, "rd wr mr mw me ac sd")],
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
                    "00005e8187da8000", Perms::from("r--p"), "0000000000000000", "008:00008", 9524160, "/usr/bin/hello   world",
                    24, 4, 4, 24, 0, 0, 24, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, "rd ex mr mw me sd")],
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
                        "000071af8c9e6000", Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "anon_inode:i915.gem",
                        16, 4, 4, 16, 16, 16, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, "rd wr mr mw me ac sd"),
                    create_smap_entry(
                        "000071af6cf0c000", Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "/memfd:wayland-shm (deleted)",
                        3560, 4, 4, 532, 108, 0, 524, 0, 8, 0, 532, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, "rd mr mw me sd"),
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
            (
                vec![
                    create_smap_entry(
                        "000071af8c9e6000", Perms::from("rw-s"), "0000000105830000", "000:00010", 1075, "anon_inode:i915.gem",
                        16, 4, 4, 16, 16, 16, 0, 0, 0, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 3, "rd wr mr mw me ac sd"),
                    create_smap_entry(
                        "0000560880413000", Perms::from("r--p"), "0000000000000000", "008:00008", 10813151, "/usr/bin/konsole",
                        180, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, ""),
                    create_smap_entry(
                        "000071af6cf0c000", Perms::from("rw-s"), "0000000000000000", "000:00001", 256481, "/memfd:wayland-shm (deleted)",
                        3560, 4, 4, 532, 108, 0, 524, 0, 8, 0, 532, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, "rd mr mw me sd"),
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
                    "ProtectionKey:          3\n",
                    "VmFlags: rd wr mr mw me ac sd\n",
                    "560880413000-560880440000 r--p 00000000 08:08 10813151                   /usr/bin/konsole\n",
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
                    "ProtectionKey:          0\n",
                    "VmFlags: rd mr mw me sd \n")
            ),
        ];

        for (expected_smap_entries, text) in data {
            let parsed = parse_smaps(text).unwrap();
            assert_eq!(expected_smap_entries, parsed.entries);
        }
    }
}
