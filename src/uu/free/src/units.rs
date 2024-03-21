use clap::ArgMatches;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
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
    pub(crate) fn from_byte(&self, byte: u64) -> f64 {
        (byte as f64) * Self::conversion_multiplier(&Self::Bytes, self)
    }

    pub(crate) fn multiplier(&self) -> u64 {
        use crate::units::UnitMultiplier::*;

        match self {
            Bytes => 1,                         // BASE
            Kilobytes => 1_000,                 // SI:10^3
            Megabytes => 1_000_000,             // SI:10^6
            Gigabytes => 1_000_000_000,         // SI:10^9
            Terabytes => 1_000_000_000_000,     // SI:10^12
            Petabytes => 1_000_000_000_000_000, // SI:10^15
            Kibibytes => 1 << 10,               // IEC:2^10
            Mebibytes => 1 << 20,               // IEC:2^20
            Gibibytes => 1 << 30,               // IEC:2^30
            Tebibytes => 1 << 40,               // IEC:2^40
            Pebibytes => 1 << 50,               // IEC:2^50
        }
    }

    // Detecting unit for `-h` and `--human` flag
    pub(crate) fn detect_readable(byte: u64) -> UnitMultiplier {
        use crate::units::UnitMultiplier::*;

        match byte {
            0..=1_000 => Bytes,
            1_001..=1_000_000 => Kilobytes,
            1_000_001..=1_000_000_000 => Mebibytes,
            1_000_000_001..=1_000_000_000_000 => Gibibytes,
            1_000_000_000_001..=1_000_000_000_000_000 => Tebibytes,
            _ => Pebibytes,
        }
    }

    fn conversion_multiplier(from: &Self, to: &Self) -> f64 {
        (from.multiplier() as f64) / (to.multiplier() as f64)
    }
}

impl Display for UnitMultiplier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use crate::units::UnitMultiplier::*;
        match self {
            Bytes => write!(f, "B"),
            Kilobytes => write!(f, "KB"),
            Megabytes => write!(f, "MB"),
            Gigabytes => write!(f, "GB"),
            Terabytes => write!(f, "TB"),
            Petabytes => write!(f, "PB"),
            Kibibytes => write!(f, "Ki"),
            Mebibytes => write!(f, "Mi"),
            Gibibytes => write!(f, "Gi"),
            Tebibytes => write!(f, "Ti"),
            Pebibytes => write!(f, "Pi"),
        }
    }
}

impl From<ArgMatches> for UnitMultiplier {
    fn from(item: ArgMatches) -> Self {
        use crate::units::UnitMultiplier::*;
        match item {
            _ if item.get_flag("kilo") => Kilobytes,
            _ if item.get_flag("mega") => Megabytes,
            _ if item.get_flag("giga") => Gigabytes,
            _ if item.get_flag("tera") => Terabytes,
            _ if item.get_flag("peta") => Petabytes,
            _ if item.get_flag("kibi") => Kibibytes,
            _ if item.get_flag("mebi") => Mebibytes,
            _ if item.get_flag("gibi") => Gibibytes,
            _ if item.get_flag("tebi") => Tebibytes,
            _ if item.get_flag("pebi") => Pebibytes,
            _ => Bytes,
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
            assert_eq!(to_unit.from_byte(from), to as f64)
        }
    }

    #[test]
    fn test_detect_readable() {
        // Value comes from my computer's `free` outputs.
        let input = [
            (007_605_510_144, UnitMultiplier::Gigabytes),
            (000_148_516_864, UnitMultiplier::Megabytes),
            (016_923_955_200, UnitMultiplier::Gigabytes),
        ];

        for (byte, unit) in input {
            assert_eq!(UnitMultiplier::detect_readable(byte), unit)
        }
    }
}
