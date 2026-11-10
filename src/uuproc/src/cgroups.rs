use std::io;

/// Represents an entry in `/proc/<pid>/cgroup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgroupMembership {
    pub hierarchy_id: u32,
    pub controllers: Vec<String>,
    pub cgroup_path: String,
}

impl TryFrom<&str> for CgroupMembership {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 3 {
            return Err(io::ErrorKind::InvalidData.into());
        }

        Ok(CgroupMembership {
            hierarchy_id: parts[0]
                .parse::<u32>()
                .map_err(|_| io::ErrorKind::InvalidData)?,
            controllers: if parts[1].is_empty() {
                vec![]
            } else {
                parts[1].split(',').map(String::from).collect()
            },
            cgroup_path: parts[2].to_string(),
        })
    }
}
