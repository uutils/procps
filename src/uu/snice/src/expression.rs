// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("failed to parse argument: '{0}'")]
    ParsingFailed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Expression {
    // The default priority is +4. (snice +4 ...)
    Increase(u32),
    Decrease(u32),
    To(u32),
}

impl TryFrom<String> for Expression {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Expression {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(stripped_value) = value.strip_prefix("-") {
            stripped_value
                .parse::<u32>()
                .map_err(|_| Error::ParsingFailed(value.into()))
                .map(Expression::Decrease)
        } else if let Some(stripped_value) = value.strip_prefix("+") {
            stripped_value
                .parse::<u32>()
                .map_err(|_| Error::ParsingFailed(value.into()))
                .map(Expression::Increase)
        } else {
            value
                .parse::<u32>()
                .map_err(|_| Error::ParsingFailed(value.into()))
                .map(Expression::To)
        }
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Increase(4)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Increase(prio) => write!(f, "+{prio}"),
            Self::Decrease(prio) => write!(f, "-{prio}"),
            Self::To(prio) => write!(f, "{prio}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        assert!(Expression::try_from("4").is_ok());
        assert!(Expression::try_from(String::from("4")).is_ok());

        assert_eq!(Expression::try_from("-4"), Ok(Expression::Decrease(4)));
        assert_eq!(Expression::try_from("+4"), Ok(Expression::Increase(4)));
        assert_eq!(Expression::try_from("4"), Ok(Expression::To(4)));

        assert_eq!(
            Expression::try_from("-4-"),
            Err(Error::ParsingFailed("-4-".into()))
        );
        assert_eq!(
            Expression::try_from("+4+"),
            Err(Error::ParsingFailed("+4+".into()))
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Expression::Decrease(4).to_string(), "-4");
        assert_eq!(Expression::Increase(4).to_string(), "+4");
        assert_eq!(Expression::To(4).to_string(), "4");
    }
}
