// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use std::{
    cmp::Ordering,
    fs,
    io::{Error, ErrorKind},
};

#[derive(Debug, Default)]
pub(crate) struct SlabInfo {
    pub(crate) meta: Vec<String>,
    pub(crate) data: Vec<(String, Vec<u64>)>,
}

impl SlabInfo {
    // parse slabinfo from /proc/slabinfo
    // need root permission
    pub fn new() -> Result<SlabInfo, Error> {
        let content = fs::read_to_string("/proc/slabinfo")?;

        Self::parse(content).ok_or(ErrorKind::Unsupported.into())
    }

    pub fn parse(content: String) -> Option<SlabInfo> {
        let mut lines: Vec<&str> = content.lines().collect();

        let _ = parse_version(lines.remove(0))?;
        let meta = parse_meta(lines.remove(0));
        let data: Vec<(String, Vec<u64>)> = lines.into_iter().filter_map(parse_data).collect();

        Some(SlabInfo { meta, data })
    }

    pub fn fetch(&self, name: &str, meta: &str) -> Option<u64> {
        // fetch meta's offset
        let offset = self.offset(meta)?;

        let (_, item) = self.data.iter().find(|(key, _)| key.eq(name))?;

        item.get(offset).copied()
    }

    pub fn names(&self) -> Vec<&String> {
        self.data.iter().map(|(k, _)| k).collect()
    }

    pub fn sort(mut self, by: char, ascending_order: bool) -> Self {
        let mut sort = |by_meta: &str| {
            if let Some(offset) = self.offset(by_meta) {
                self.data.sort_by(|(_, data1), (_, data2)| {
                    match (data1.get(offset), data2.get(offset)) {
                        (Some(v1), Some(v2)) => {
                            if ascending_order {
                                v1.cmp(v2)
                            } else {
                                v2.cmp(v1)
                            }
                        }
                        _ => Ordering::Equal,
                    }
                });
            }
        };

        match by {
            // <active_objs>
            'a' => sort("active_objs"),
            // <objperslab>
            'b' => sort("objperslab"),
            // <objsize> Maybe cache size I guess?
            // TODO: Check is <objsize>
            'c' => sort("objsize"),
            // <num_slabs>
            'l' => sort("num_slabs"),
            // <active_slabs>
            'v' => sort("active_slabs"),
            // name, sort by lexicographical order
            'n' => self.data.sort_by(|(name1, _), (name2, _)| {
                if ascending_order {
                    name1.cmp(name2)
                } else {
                    name2.cmp(name1)
                }
            }),
            // <pagesperslab>
            'p' => sort("pagesperslab"),
            // <objsize>
            's' => sort("objsize"),
            // sort by cache utilization
            'u' => {
                let offset_active_objs = self.offset("active_objs");
                let offset_num_objs = self.offset("num_objs");

                if let (Some(offset_active_objs), Some(offset_num_objs)) =
                    (offset_active_objs, offset_num_objs)
                {
                    self.data.sort_by(|(_, data1), (_, data2)| {
                        let cu = |active_jobs: Option<&u64>, num_jobs: Option<&u64>| match (
                            active_jobs,
                            num_jobs,
                        ) {
                            (Some(active_jobs), Some(num_jobs)) => {
                                Some((*active_jobs as f64) / (*num_jobs as f64))
                            }
                            _ => None,
                        };
                        let cu1 = cu(data1.get(offset_active_objs), data1.get(offset_num_objs));
                        let cu2 = cu(data2.get(offset_active_objs), data2.get(offset_num_objs));

                        if let (Some(cu1), Some(cu2)) = (cu1, cu2) {
                            let result = if ascending_order {
                                cu1.partial_cmp(&cu2)
                            } else {
                                cu2.partial_cmp(&cu1)
                            };
                            match result {
                                Some(ord) => ord,
                                None => Ordering::Equal,
                            }
                        } else {
                            Ordering::Equal
                        }
                    })
                }
            }

            // <num_objs>
            // Default branch : `o`
            _ => sort("num_objs"),
        }

        self
    }

    fn offset(&self, meta: &str) -> Option<usize> {
        self.meta.iter().position(|it| it.eq(meta))
    }

    /////////////////////////////////// helpers ///////////////////////////////////

    #[inline]
    fn total(&self, meta: &str) -> u64 {
        let Some(offset) = self.offset(meta) else {
            return 0;
        };

        self.data
            .iter()
            .filter_map(|(_, data)| data.get(offset))
            .sum::<u64>()
    }

    pub fn object_minimum(&self) -> u64 {
        let Some(offset) = self.offset("objsize") else {
            return 0;
        };

        match self
            .data
            .iter()
            .filter_map(|(_, data)| data.get(offset))
            .min()
        {
            Some(min) => *min,
            None => 0,
        }
    }

    pub fn object_maximum(&self) -> u64 {
        let Some(offset) = self.offset("objsize") else {
            return 0;
        };

        match self
            .data
            .iter()
            .filter_map(|(_, data)| data.get(offset))
            .max()
        {
            Some(max) => *max,
            None => 0,
        }
    }

    pub fn object_avg(&self) -> u64 {
        let Some(offset) = self.offset("objsize") else {
            return 0;
        };

        let iter = self.data.iter().filter_map(|(_, data)| data.get(offset));

        let count = iter.clone().count();
        let sum = iter.sum::<u64>();

        if count == 0 {
            0
        } else {
            (sum) / (count as u64)
        }
    }

    pub fn total_active_objs(&self) -> u64 {
        self.total("active_objs")
    }

    pub fn total_objs(&self) -> u64 {
        self.total("num_objs")
    }

    pub fn total_active_slabs(&self) -> u64 {
        self.total("active_slabs")
    }

    pub fn total_slabs(&self) -> u64 {
        self.total("num_slabs")
    }

    pub fn total_active_size(&self) -> u64 {
        self.names()
            .iter()
            .map(|name| {
                self.fetch(name, "active_objs").unwrap_or_default()
                    * self.fetch(name, "objsize").unwrap_or_default()
            })
            .sum::<u64>()
    }

    pub fn total_size(&self) -> u64 {
        self.names()
            .iter()
            .map(|name| {
                self.fetch(name, "num_objs").unwrap_or_default()
                    * self.fetch(name, "objsize").unwrap_or_default()
            })
            .sum::<u64>()
    }

    pub fn total_active_cache(&self) -> u64 {
        self.names()
            .iter()
            .map(|name| {
                self.fetch(name, "objsize").unwrap_or_default()
                    * self.fetch(name, "active_objs").unwrap_or_default()
            })
            .sum::<u64>()
    }

    pub fn total_cache(&self) -> u64 {
        self.names()
            .iter()
            .map(|name| {
                self.fetch(name, "objsize").unwrap_or_default()
                    * self.fetch(name, "num_objs").unwrap_or_default()
            })
            .sum::<u64>()
    }
}

pub(crate) fn parse_version(line: &str) -> Option<String> {
    line.replace(':', " ")
        .split_whitespace()
        .last()
        .map(String::from)
}

pub(crate) fn parse_meta(line: &str) -> Vec<String> {
    line.replace(['#', ':'], " ")
        .split_whitespace()
        .filter(|it| it.starts_with('<') && it.ends_with('>'))
        .map(|it| it.replace(['<', '>'], ""))
        .collect()
}

pub(crate) fn parse_data(line: &str) -> Option<(String, Vec<u64>)> {
    let split: Vec<String> = line
        .replace(':', " ")
        .split_whitespace()
        .map(String::from)
        .collect();

    split.first().map(|name| {
        (
            name.to_string(),
            split
                .clone()
                .into_iter()
                .flat_map(|it| it.parse::<u64>())
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let test = "slabinfo - version: 2.1";
        assert_eq!("2.1", parse_version(test).unwrap())
    }

    #[test]
    fn test_parse_meta() {
        let test="# name            <active_objs> <num_objs> <objsize> <objperslab> <pagesperslab> : tunables <limit> <batchcount> <sharedfactor> : slabdata <active_slabs> <num_slabs> <sharedavail>";

        let result = parse_meta(test);

        assert_eq!(
            result,
            [
                "active_objs",
                "num_objs",
                "objsize",
                "objperslab",
                "pagesperslab",
                "limit",
                "batchcount",
                "sharedfactor",
                "active_slabs",
                "num_slabs",
                "sharedavail"
            ]
        )
    }

    #[test]
    fn test_parse_data() {
        // Success case

        let test = "nf_conntrack_expect      0      0    208   39    2 : tunables    0    0    0 : slabdata      0      0      0";
        let (name, value) = parse_data(test).unwrap();

        assert_eq!(name, "nf_conntrack_expect");
        assert_eq!(value, [0, 0, 208, 39, 2, 0, 0, 0, 0, 0, 0]);

        // Fail case
        let test =
            "0      0    208   39    2 : tunables    0    0    0 : slabdata      0      0      0";
        let (name, _value) = parse_data(test).unwrap();

        assert_ne!(name, "nf_conntrack_expect");
    }

    #[test]
    fn test_parse() {
        let test = include_str!("../../../../tests/fixtures/slabtop/data.txt");
        let result = SlabInfo::parse(test.into()).unwrap();

        assert_eq!(result.fetch("nf_conntrack_expect", "objsize").unwrap(), 208);
        assert_eq!(
            result.fetch("dmaengine-unmap-2", "active_slabs").unwrap(),
            16389
        );
    }
}
