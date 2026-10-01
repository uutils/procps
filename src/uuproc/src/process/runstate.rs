use std::{
    fmt::{self, Display, Formatter},
    io,
};

/// State of process
/// https://www.man7.org/linux/man-pages//man5/proc_pid_stat.5.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunState {
    /// `R`, running
    Running,
    /// `S`, sleeping
    Sleeping,
    /// `D`, sleeping in an uninterruptible wait
    UninterruptibleWait,
    /// `Z`, zombie
    Zombie,
    /// `T`, stopped (on a signal)
    Stopped,
    /// `t`, tracing stop
    TraceStopped,
    /// `X`, dead
    Dead,
    /// `I`, idle
    Idle,
}

impl Display for RunState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "R"),
            Self::Sleeping => write!(f, "S"),
            Self::UninterruptibleWait => write!(f, "D"),
            Self::Zombie => write!(f, "Z"),
            Self::Stopped => write!(f, "T"),
            Self::TraceStopped => write!(f, "t"),
            Self::Dead => write!(f, "X"),
            Self::Idle => write!(f, "I"),
        }
    }
}

impl TryFrom<char> for RunState {
    type Error = io::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'R' => Ok(Self::Running),
            'S' => Ok(Self::Sleeping),
            'D' => Ok(Self::UninterruptibleWait),
            'Z' => Ok(Self::Zombie),
            'T' => Ok(Self::Stopped),
            't' => Ok(Self::TraceStopped),
            'X' => Ok(Self::Dead),
            'I' => Ok(Self::Idle),
            _ => Err(io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl TryFrom<&str> for RunState {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        Self::try_from(
            value
                .chars()
                .nth(0)
                .ok_or::<io::Error>(io::ErrorKind::InvalidInput.into())?,
        )
    }
}
