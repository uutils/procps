// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::convert::Infallible;
use thiserror::Error as TError;

#[derive(Debug, TError, PartialEq, Eq)]
pub enum Error {
    #[error("empty value")]
    EmptyValue,

    #[error("parsing failed")]
    ParsingFailed,
}

/// Parsing _**optional**_ key-value arguments
///
/// There are two formats
///
/// - `cmd` -> key: `cmd`, value: None
/// - `cmd=CMD` -> key: `cmd`, value: `CMD`
///
/// Other formats can also be parsed:
///
/// - `cmd=` -> key: `cmd`, value: (empty, no space there)
/// - `cmd=abcd123~~~~` -> key: `cmd`, value: `abcd123~~~~`
/// - `cmd======?` -> key: `cmd`, value: `=====?`
#[derive(Debug, Clone)]
pub struct OptionalKeyValue {
    key: String,
    value: Option<String>,
}

impl OptionalKeyValue {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<String>,
    {
        let value: String = value.into();

        if let Some((key, value)) = value.split_once("=") {
            Self {
                key: key.into(),
                value: Some(value.into()),
            }
        } else {
            Self {
                key: value,
                value: None,
            }
        }
    }

    pub fn with_key<T>(key: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            key: key.into(),
            value: None,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Option<String> {
        &self.value
    }

    pub fn is_value_empty(&self) -> bool {
        self.value.is_none()
    }

    pub fn try_get<T: std::str::FromStr>(&self) -> Result<T, Error> {
        let Some(ref value) = self.value else {
            return Err(Error::EmptyValue);
        };

        value.parse::<T>().map_err(|_| Error::ParsingFailed)
    }
}

// clap value parser wrapper
pub(crate) fn parser(value: &str) -> Result<OptionalKeyValue, Infallible> {
    Ok(OptionalKeyValue::new(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[inline(always)]
    fn new<T>(value: T) -> OptionalKeyValue
    where
        T: Into<String>,
    {
        OptionalKeyValue::new(value)
    }

    #[test]
    fn test_parsing() {
        assert!(new("value").is_value_empty());
        assert!(!new("value=").is_value_empty());
        assert!(!new("value=v").is_value_empty());
        assert!(!new("value=?:").is_value_empty());

        assert!(OptionalKeyValue::with_key("key").key().eq("key"));
    }

    #[test]
    fn test_get_key() {
        assert_eq!(new("value").key(), "value");
        assert_eq!(new("value=").key(), "value");
        assert_eq!(new("value=?").key(), "value");
    }

    #[test]
    fn test_get_value() {
        // String test
        assert_eq!(new("value").try_get::<String>(), Err(Error::EmptyValue));
        assert_eq!(new("value=").try_get::<String>(), Ok("".into()));
        assert_eq!(new("value=?").try_get::<String>(), Ok("?".into()));

        // Number test
        assert_eq!(new("value").try_get::<usize>(), Err(Error::EmptyValue));
        assert_eq!(new("value=0").try_get::<usize>(), Ok(0));
        assert_eq!(new("value=0").try_get::<i128>(), Ok(0));
        assert_eq!(new("value=-1").try_get::<i128>(), Ok(-1));
        assert_eq!(new("value=0").try_get::<u128>(), Ok(0));
        assert_eq!(new("value=-1").try_get::<u128>(), Err(Error::ParsingFailed));
    }
}
