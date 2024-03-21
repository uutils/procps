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

const fn pow(base: u64, exponent: u32) -> u64 {
    if exponent == 0 {
        1
    } else if exponent % 2 == 0 {
        let half_pow = pow(base, exponent / 2);
        half_pow * half_pow
    } else {
        base * pow(base, exponent - 1)
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
