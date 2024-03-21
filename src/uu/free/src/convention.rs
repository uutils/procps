use clap::{ArgMatches};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub(crate) enum UnitMultiplier {
    Bytes,     // BASE UNIT
    Kilobytes, // SI:10^3
    Megabytes, // SI:10^6
    Gigabytes, // SI:10^9
    Terabytes, // SI:10^12
    Petabytes, // SI:10^15
    Kibibytes, // IEC:2^10
    Mebibytes, // IEC:2^20
    Gibibytes, // IEC:2^30
    Tebibytes, // IEC:2^40
    Pebibytes, // IEC:2^50
}

impl UnitMultiplier {
    pub(crate) fn from_bytes_to(byte: u64, multiplier: UnitMultiplier) -> f64 {
        (byte as f64) * Self::conversion_multiplier(Self::Bytes, multiplier)
    }

    pub(crate) fn multiplier(self) -> u64 {
        use crate::convention::UnitMultiplier::*;

        match self {
            Bytes => 1,                         // BASE
            Kilobytes => 1_000,                 // SI:10^3
            Megabytes => 1_000_000,             // SI:10^6
            Gigabytes => 1_000_000_000,         // SI:10^9
            Terabytes => 1_000_000_000_000,     // SI:10^12
            Petabytes => 1_000_000_000_000_000, // SI:10^15
            Kibibytes => 2 << 10,               // IEC:2^10
            Mebibytes => 2 << 20,               // IEC:2^20
            Gibibytes => 2 << 30,               // IEC:2^30
            Tebibytes => 2 << 40,               // IEC:2^40
            Pebibytes => 2 << 50,               // IEC:2^50
        }
    }

    fn conversion_multiplier(from: Self, to: Self) -> f64 {
        (from.multiplier() as f64) / (to.multiplier() as f64)
    }
}

impl Display for UnitMultiplier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use crate::convention::UnitMultiplier::*;
        match self {
            Bytes => write!(f, "B"),
            Kilobytes => write!(f, "Ki"),
            Megabytes => write!(f, "Mi"),
            Gigabytes => write!(f, "Gi"),
            Terabytes => write!(f, "Ti"),
            Petabytes => write!(f, "Pi"),
            Kibibytes => write!(f, "KB"),
            Mebibytes => write!(f, "MB"),
            Gibibytes => write!(f, "GB"),
            Tebibytes => write!(f, "TB"),
            Pebibytes => write!(f, "PB"),
        }
    }
}

impl From<ArgMatches> for UnitMultiplier {
    fn from(item: ArgMatches) -> Self {
        use crate::convention::UnitMultiplier::*;
        match item {
            _ if item.get_flag("bytes") => Bytes,
            _ if item.get_flag("kilo") => Kilobytes,
            _ if item.get_flag("mega") => Megabytes,
            _ if item.get_flag("giga") => Gigabytes,
            _ if item.get_flag("tera") => Terabytes,
            _ if item.get_flag("peta") => Petabytes,
            _ if item.get_flag("kibi") => Kibibytes,
            _ if item.get_flag("mebi") => Mebibytes,
            _ if item.get_flag("gibi") => Gibibytes,
            _ if item.get_flag("tebi") => Tebibytes,
            _ => Pebibytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        let input = [
            (
                (UnitMultiplier::Bytes, 1_000_000_000_000_000 as u64),
                (UnitMultiplier::Petabytes, 1 as u64),
            ),
            ((UnitMultiplier::Bytes, 1), (UnitMultiplier::Bytes, 1)),
            (
                (UnitMultiplier::Bytes, 1_000),
                (UnitMultiplier::Kilobytes, 1),
            ),
            (
                (UnitMultiplier::Bytes, 1_000_000),
                (UnitMultiplier::Megabytes, 1),
            ),
        ];

        for ((_, from), (to_unit, to)) in input {
            assert_eq!(UnitMultiplier::from_bytes_to(from, to_unit), to as f64)
        }
    }
}
