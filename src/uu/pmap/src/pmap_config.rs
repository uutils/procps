// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use dirs::home_dir;
use std::io::Error;
use std::path::PathBuf;

pub mod pmap_field_name {
    pub const ADDRESS: &str = "Address";
    pub const PERM: &str = "Perm";
    pub const OFFSET: &str = "Offset";
    pub const DEVICE: &str = "Device";
    pub const INODE: &str = "Inode";
    pub const SIZE: &str = "Size";
    pub const KERNEL_PAGE_SIZE: &str = "KernelPageSize";
    pub const MMU_PAGE_SIZE: &str = "MMUPageSize";
    pub const RSS: &str = "Rss";
    pub const PSS: &str = "Pss";
    pub const PSS_DIRTY: &str = "Pss_Dirty";
    pub const SHARED_CLEAN: &str = "Shared_Clean";
    pub const SHARED_DIRTY: &str = "Shared_Dirty";
    pub const PRIVATE_CLEAN: &str = "Private_Clean";
    pub const PRIVATE_DIRTY: &str = "Private_Dirty";
    pub const REFERENCED: &str = "Referenced";
    pub const ANONYMOUS: &str = "Anonymous";
    pub const KSM: &str = "KSM";
    pub const LAZY_FREE: &str = "LazyFree";
    pub const ANON_HUGE_PAGES: &str = "AnonHugePages";
    pub const SHMEM_PMD_MAPPED: &str = "ShmemPmdMapped";
    pub const FILE_PMD_MAPPED: &str = "FilePmdMapped";
    pub const SHARED_HUGETLB: &str = "Shared_Hugetlb";
    pub const PRIVATE_HUGETLB: &str = "Private_Hugetlb";
    pub const SWAP: &str = "Swap";
    pub const SWAP_PSS: &str = "SwapPss";
    pub const LOCKED: &str = "Locked";
    pub const THP_ELIGIBLE: &str = "THPeligible";
    pub const PROTECTION_KEY: &str = "ProtectionKey";
    pub const VMFLAGS: &str = "VmFlags";
    pub const MAPPING: &str = "Mapping";
}

// Represents the configuration for enabling specific fields.
// Note: Address field is always enabled.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PmapConfig {
    // [Fields Display] category
    pub perm: bool,
    pub offset: bool,
    pub device: bool,
    pub inode: bool,
    pub size: bool,
    pub kernel_page_size: bool,
    pub mmu_page_size: bool,
    pub rss: bool,
    pub pss: bool,
    pub pss_dirty: bool,
    pub shared_clean: bool,
    pub shared_dirty: bool,
    pub private_clean: bool,
    pub private_dirty: bool,
    pub referenced: bool,
    pub anonymous: bool,
    pub ksm: bool,
    pub lazy_free: bool,
    pub anon_huge_pages: bool,
    pub shmem_pmd_mapped: bool,
    pub file_pmd_mapped: bool,
    pub shared_hugetlb: bool,
    pub private_hugetlb: bool,
    pub swap: bool,
    pub swap_pss: bool,
    pub locked: bool,
    pub thp_eligible: bool,
    pub protection_key: bool,
    pub vmflags: bool,
    pub mapping: bool,
    // [Mapping] category
    pub show_path: bool,
    // Misc
    pub quiet: bool,
    pub custom_format_enabled: bool,
}

impl PmapConfig {
    pub fn get_field_list(&self) -> [&'static str; 29] {
        // Note: Address and Mapping are treated separately from other fields.
        [
            pmap_field_name::PERM,
            pmap_field_name::OFFSET,
            pmap_field_name::DEVICE,
            pmap_field_name::INODE,
            pmap_field_name::SIZE,
            pmap_field_name::KERNEL_PAGE_SIZE,
            pmap_field_name::MMU_PAGE_SIZE,
            pmap_field_name::RSS,
            pmap_field_name::PSS,
            pmap_field_name::PSS_DIRTY,
            pmap_field_name::SHARED_CLEAN,
            pmap_field_name::SHARED_DIRTY,
            pmap_field_name::PRIVATE_CLEAN,
            pmap_field_name::PRIVATE_DIRTY,
            pmap_field_name::REFERENCED,
            pmap_field_name::ANONYMOUS,
            pmap_field_name::KSM,
            pmap_field_name::LAZY_FREE,
            pmap_field_name::ANON_HUGE_PAGES,
            pmap_field_name::SHMEM_PMD_MAPPED,
            pmap_field_name::FILE_PMD_MAPPED,
            pmap_field_name::SHARED_HUGETLB,
            pmap_field_name::PRIVATE_HUGETLB,
            pmap_field_name::SWAP,
            pmap_field_name::SWAP_PSS,
            pmap_field_name::LOCKED,
            pmap_field_name::THP_ELIGIBLE,
            pmap_field_name::PROTECTION_KEY,
            pmap_field_name::VMFLAGS,
        ]
    }

    pub fn needs_footer(&self, field_name: &str) -> bool {
        !matches!(
            field_name,
            pmap_field_name::ADDRESS
                | pmap_field_name::PERM
                | pmap_field_name::OFFSET
                | pmap_field_name::DEVICE
                | pmap_field_name::INODE
                | pmap_field_name::VMFLAGS
                | pmap_field_name::MAPPING
        )
    }

    pub fn is_enabled(&self, field_name: &str) -> bool {
        match field_name {
            pmap_field_name::PERM => self.perm,
            pmap_field_name::OFFSET => self.offset,
            pmap_field_name::DEVICE => self.device,
            pmap_field_name::INODE => self.inode,
            pmap_field_name::SIZE => self.size,
            pmap_field_name::KERNEL_PAGE_SIZE => self.kernel_page_size,
            pmap_field_name::MMU_PAGE_SIZE => self.mmu_page_size,
            pmap_field_name::RSS => self.rss,
            pmap_field_name::PSS => self.pss,
            pmap_field_name::PSS_DIRTY => self.pss_dirty,
            pmap_field_name::SHARED_CLEAN => self.shared_clean,
            pmap_field_name::SHARED_DIRTY => self.shared_dirty,
            pmap_field_name::PRIVATE_CLEAN => self.private_clean,
            pmap_field_name::PRIVATE_DIRTY => self.private_dirty,
            pmap_field_name::REFERENCED => self.referenced,
            pmap_field_name::ANONYMOUS => self.anonymous,
            pmap_field_name::KSM => self.ksm,
            pmap_field_name::LAZY_FREE => self.lazy_free,
            pmap_field_name::ANON_HUGE_PAGES => self.anon_huge_pages,
            pmap_field_name::SHMEM_PMD_MAPPED => self.shmem_pmd_mapped,
            pmap_field_name::FILE_PMD_MAPPED => self.file_pmd_mapped,
            pmap_field_name::SHARED_HUGETLB => self.shared_hugetlb,
            pmap_field_name::PRIVATE_HUGETLB => self.private_hugetlb,
            pmap_field_name::SWAP => self.swap,
            pmap_field_name::SWAP_PSS => self.swap_pss,
            pmap_field_name::LOCKED => self.locked,
            pmap_field_name::THP_ELIGIBLE => self.thp_eligible,
            pmap_field_name::PROTECTION_KEY => self.protection_key,
            pmap_field_name::VMFLAGS => self.vmflags,
            pmap_field_name::MAPPING => self.mapping,
            _ => false,
        }
    }

    fn set_field(&mut self, field_name: &str, val: bool) {
        match field_name {
            pmap_field_name::PERM => self.perm = val,
            pmap_field_name::OFFSET => self.offset = val,
            pmap_field_name::DEVICE => self.device = val,
            pmap_field_name::INODE => self.inode = val,
            pmap_field_name::SIZE => self.size = val,
            pmap_field_name::KERNEL_PAGE_SIZE => self.kernel_page_size = val,
            pmap_field_name::MMU_PAGE_SIZE => self.mmu_page_size = val,
            pmap_field_name::RSS => self.rss = val,
            pmap_field_name::PSS => self.pss = val,
            pmap_field_name::PSS_DIRTY => self.pss_dirty = val,
            pmap_field_name::SHARED_CLEAN => self.shared_clean = val,
            pmap_field_name::SHARED_DIRTY => self.shared_dirty = val,
            pmap_field_name::PRIVATE_CLEAN => self.private_clean = val,
            pmap_field_name::PRIVATE_DIRTY => self.private_dirty = val,
            pmap_field_name::REFERENCED => self.referenced = val,
            pmap_field_name::ANONYMOUS => self.anonymous = val,
            pmap_field_name::KSM => self.ksm = val,
            pmap_field_name::LAZY_FREE => self.lazy_free = val,
            pmap_field_name::ANON_HUGE_PAGES => self.anon_huge_pages = val,
            pmap_field_name::SHMEM_PMD_MAPPED => self.shmem_pmd_mapped = val,
            pmap_field_name::FILE_PMD_MAPPED => self.file_pmd_mapped = val,
            pmap_field_name::SHARED_HUGETLB => self.shared_hugetlb = val,
            pmap_field_name::PRIVATE_HUGETLB => self.private_hugetlb = val,
            pmap_field_name::SWAP => self.swap = val,
            pmap_field_name::SWAP_PSS => self.swap_pss = val,
            pmap_field_name::LOCKED => self.locked = val,
            pmap_field_name::THP_ELIGIBLE => self.thp_eligible = val,
            pmap_field_name::PROTECTION_KEY => self.protection_key = val,
            pmap_field_name::VMFLAGS => self.vmflags = val,
            pmap_field_name::MAPPING => self.mapping = val,
            _ => (),
        }
    }

    pub fn enable_field(&mut self, field_name: &str) {
        self.set_field(field_name, true);
    }

    pub fn disable_field(&mut self, field_name: &str) {
        self.set_field(field_name, false);
    }

    // Preset for more-extended option
    pub fn set_more_extended(&mut self) {
        self.custom_format_enabled = true;
        self.perm = true;
        self.offset = true;
        self.device = true;
        self.inode = true;
        self.size = true;
        self.rss = true;
        self.pss = true;
        self.pss_dirty = true;
        self.referenced = true;
        self.anonymous = true;
        self.ksm = true;
        self.lazy_free = true;
        self.shmem_pmd_mapped = true;
        self.file_pmd_mapped = true;
        self.shared_hugetlb = true;
        self.private_hugetlb = true;
        self.swap = true;
        self.swap_pss = true;
        self.locked = true;
        self.thp_eligible = true;
        self.protection_key = true;
        self.mapping = true;
    }

    // Preset for most-extended option
    pub fn set_most_extended(&mut self) {
        self.custom_format_enabled = true;
        self.set_more_extended();
        self.kernel_page_size = true;
        self.mmu_page_size = true;
        self.shared_clean = true;
        self.shared_dirty = true;
        self.private_clean = true;
        self.private_dirty = true;
        self.anon_huge_pages = true;
        self.vmflags = true;
    }

    pub fn read_rc(&mut self, path: &PathBuf) -> Result<(), Error> {
        self.custom_format_enabled = true;

        let contents = std::fs::read_to_string(path)?;

        let mut in_field_display = false;
        let mut in_mapping = false;

        for line in contents.lines() {
            let line = line.trim_ascii();
            if line.starts_with("#") || line.len() == 0 {
                continue;
            }

            // The leftmost category on the line is recoginized.
            if line.starts_with("[Fields Display]") {
                in_field_display = true;
                in_mapping = false;
                continue;
            } else if line.starts_with("[Mapping]") {
                in_field_display = false;
                in_mapping = true;
                continue;
            }

            if in_field_display {
                self.enable_field(line);
            } else if in_mapping {
                if line == "ShowPath" {
                    self.show_path = true;
                }
            }
        }

        Ok(())
    }
}

pub fn create_rc(path: &PathBuf) -> Result<(), Error> {
    let contents = "# pmap's Config File\n".to_string()
        + "\n"
        + "# All the entries are case sensitive.\n"
        + "# Unsupported entries are ignored!\n"
        + "\n"
        + "[Fields Display]\n"
        + "\n"
        + "# To enable a field uncomment its entry\n"
        + "\n"
        + "#Perm\n"
        + "#Offset\n"
        + "#Device\n"
        + "#Inode\n"
        + "#Size\n"
        + "#Rss\n"
        + "#Pss\n"
        + "#Shared_Clean\n"
        + "#Shared_Dirty\n"
        + "#Private_Clean\n"
        + "#Private_Dirty\n"
        + "#Referenced\n"
        + "#Anonymous\n"
        + "#AnonHugePages\n"
        + "#Swap\n"
        + "#KernelPageSize\n"
        + "#MMUPageSize\n"
        + "#Locked\n"
        + "#VmFlags\n"
        + "#Mapping\n"
        + "\n"
        + "[Mapping]\n"
        + "\n"
        + "# to show paths in the mapping column uncomment the following line\n"
        + "#ShowPath\n"
        + "\n";

    std::fs::write(path, contents)?;

    Ok(())
}

pub fn get_rc_default_path() -> PathBuf {
    let mut path = home_dir().expect("home directory should not be None");
    path.push(".pmaprc");
    path
}

pub fn get_rc_default_path_str() -> &'static str {
    "~/.pmaprc"
}
