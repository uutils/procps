// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::time::Duration;

pub(crate) struct TuiStat {
    pub cpu_graph_mode: CpuGraphMode,
    pub cpu_value_mode: CpuValueMode,
    pub memory_graph_mode: MemoryGraphMode,
    pub list_offset: usize,
    pub delay: Duration,
}

impl TuiStat {
    pub fn new() -> Self {
        Self {
            cpu_graph_mode: CpuGraphMode::default(),
            cpu_value_mode: CpuValueMode::default(),
            memory_graph_mode: MemoryGraphMode::default(),
            list_offset: 0,
            delay: Duration::from_millis(1500), // 1.5s
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum CpuGraphMode {
    #[default]
    Block,
    Bar,
    Sum,
    Hide,
}

impl CpuGraphMode {
    pub fn next(&self) -> CpuGraphMode {
        match self {
            CpuGraphMode::Block => CpuGraphMode::Hide,
            CpuGraphMode::Hide => CpuGraphMode::Sum,
            CpuGraphMode::Sum => CpuGraphMode::Bar,
            CpuGraphMode::Bar => CpuGraphMode::Block,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum CpuValueMode {
    #[default]
    PerCore,
    Sum,
}

impl CpuValueMode {
    pub fn next(&self) -> CpuValueMode {
        match self {
            CpuValueMode::PerCore => CpuValueMode::Sum,
            CpuValueMode::Sum => CpuValueMode::PerCore,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub enum MemoryGraphMode {
    #[default]
    Block,
    Bar,
    Sum,
    Hide,
}

impl MemoryGraphMode {
    pub fn next(&self) -> MemoryGraphMode {
        match self {
            MemoryGraphMode::Block => MemoryGraphMode::Hide,
            MemoryGraphMode::Hide => MemoryGraphMode::Sum,
            MemoryGraphMode::Sum => MemoryGraphMode::Bar,
            MemoryGraphMode::Bar => MemoryGraphMode::Block,
        }
    }
}
