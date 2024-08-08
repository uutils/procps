// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::convert::Infallible;

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

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Option<String> {
        &self.value
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
    fn test_get_key() {
        assert_eq!(new("value").key(), "value");
        assert_eq!(new("value=").key(), "value");
        assert_eq!(new("value=?").key(), "value");
    }
}
