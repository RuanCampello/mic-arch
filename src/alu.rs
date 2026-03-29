//! Arithmetic logic unit `mic-1` implementation.

/// The arithmetic logic unit of `mic-1`.
/// This performs arithmetic and logic (duh) operations on `A` and `B`
/// according to the [control](self::AluInstruction) signals.
///
/// Although this keeps no state, we use for scope purposes.
pub(crate) struct Alu;

/// The `ALU` returns the core result `s`, shifted output `sd`, flags `n`/`z`, and the carry (vai-um).
#[derive(Debug)]
pub struct AluResult {
    pub s: u32,
    pub sd: u32,
    pub carry: bool,
    pub n: bool,
    pub z: bool,
}

/// The `ALU` receives control instructions with the following 8-bit format (MSB to LSB):
///
/// | Bit | Signal | Description |
/// |-----|--------|-------------|
/// | 7   | SLL8   | if set, `sd = s << 8` (after core ALU); mutually exclusive with `SRA1` |
/// | 6   | SRA1   | if set, `sd = (s >> 1) \| (s & 0x80000000)`; mutually exclusive with `SLL8` |
/// | 5   | F0     | selects ALU operation |
/// | 4   | F1     | selects ALU operation |
/// | 3   | ENA    | enables input `a`. if `0`, `a` is treated as `0`. |
/// | 2   | ENB    | enables input `b`. if `0`, `b` is treated as `0`. |
/// | 1   | INVA   | inverts input `a` before the ALU operation. |
/// | 0   | INC    | forces carry-in = 1, effectively adding `+1` to the result. |
///
/// The `F0` and `F1` bits determine the core ALU operation:
///
/// | F0 | F1 | Operation |
/// |----|----|-----------|
/// | 0  | 0  | `A & B` (logic AND) |
/// | 0  | 1  | `A \| B` (logic OR) |
/// | 1  | 0  | `!B` (logic NOT) |
/// | 1  | 1  | `A + B` (arithmetic ADD) |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AluInstruction {
    pub sll8: bool,
    pub sra1: bool,
    pub f0: bool,
    pub f1: bool,
    pub ena: bool,
    pub enb: bool,
    pub inva: bool,
    pub inc: bool,
}

impl AluInstruction {
    /// `SLL8` and `SRA1` must not both be set.
    #[inline]
    pub const fn is_valid(self) -> bool {
        !(self.sll8 && self.sra1)
    }
}

#[derive(Debug, PartialEq)]
pub enum AluParseError {
    InvalidLength(usize),
    NonBinaryChar,
    IntParse(std::num::ParseIntError),
}

/// Respectively 'a' and 'b' used during ALU calculation.
pub struct Inputs {
    pub a: u32,
    pub b: u32,
}

impl Alu {
    pub fn execute(a: u32, b: u32, control: AluInstruction) -> (Inputs, AluResult) {
        // verify which inputs are enabled
        let mut a = if control.ena { a } else { 0 };
        let b = if control.enb { b } else { 0 };

        // verify if we need to invert A BEFORE the operation
        if control.inva {
            a = !a;
        }

        let mut carry = false;

        // this follows the specification interpretation of bits for f0, f1.
        let mut result = match (control.f0, control.f1) {
            (false, false) => a & b,
            (false, true) => a | b,
            (true, false) => !b,
            (true, true) => {
                let (s, c) = a.overflowing_add(b);
                carry = c;
                s
            }
        };

        // verify if we need to increment the result
        // here we need to take a little care of inc because it can overflow too
        // in that case, we also need to flip the carry
        if control.inc {
            let (s, c) = result.overflowing_add(1);
            result = s;
            carry |= c;
        }

        // post-process: SLL8 / SRA1 on the core output `s` (invalid if both shift bits are set)
        if control.sll8 && control.sra1 {
            return (
                Inputs { a, b },
                AluResult {
                    s: result,
                    sd: 0,
                    carry,
                    n: false,
                    z: false,
                },
            );
        }

        // shifting happens after the core ALU output `s` is computed
        let sd = if control.sll8 {
            result.wrapping_shl(8)
        } else if control.sra1 {
            (result >> 1) | (result & 0x8000_0000)
        } else {
            result
        };

        // flags from `sd`
        let n = (sd >> 31) != 0;
        let z = sd == 0;

        (
            Inputs { a, b },
            AluResult {
                s: result,
                sd,
                carry,
                n,
                z,
            },
        )
    }
}

impl std::fmt::Display for AluInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bit = |b: bool| if b { '1' } else { '0' };
        write!(
            f,
            "{}{}{}{}{}{}{}{}",
            bit(self.sll8),
            bit(self.sra1),
            bit(self.f0),
            bit(self.f1),
            bit(self.ena),
            bit(self.enb),
            bit(self.inva),
            bit(self.inc)
        )
    }
}

impl From<u8> for AluInstruction {
    fn from(bits: u8) -> Self {
        Self {
            sll8: bits & 0x80 != 0,
            sra1: bits & 0x40 != 0,
            f0: bits & 0x20 != 0,
            f1: bits & 0x10 != 0,
            ena: bits & 0x08 != 0,
            enb: bits & 0x04 != 0,
            inva: bits & 0x02 != 0,
            inc: bits & 0x01 != 0,
        }
    }
}

impl std::str::FromStr for AluInstruction {
    type Err = AluParseError;

    /// Accepts exactly **6** or **8** binary digits. Six-digit lines set `SLL8` and `SRA1` to `0`
    /// (legacy program files); eight-digit lines use the full control word.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Here validating binary input
        let s = s.trim();
        if s.len() != 6 && s.len() != 8 {
            return Err(AluParseError::InvalidLength(s.len()));
        }
        if !s.chars().all(|c| c == '0' || c == '1') {
            return Err(AluParseError::NonBinaryChar);
        }

        // Converting:
        // FROM: string of 6 or 8 bits
        // TO: u8
        let bits = u8::from_str_radix(s, 2)?;
        Ok(Self::from(bits))
    }
}

impl std::fmt::Display for AluParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                writeln!(f, "expected exactly 6 or 8 bits, but found: {length} >:(")
            }
            Self::NonBinaryChar => writeln!(f, "expected only binary digits :("),
            Self::IntParse(parse) => writeln!(f, "{parse}"),
        }
    }
}

impl From<std::num::ParseIntError> for AluParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::IntParse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_operation() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: false,
            f1: false,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(0b1100, 0b1010, ctrl);

        // A: 1100
        // B: 1010
        // &: 1000
        assert_eq!(res.s, 0b1000);
        assert_eq!(res.sd, res.s);
        assert!(!res.n);
        assert!(!res.z);
        assert!(!res.carry);
    }

    #[test]
    fn or_operation() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: false,
            f1: true,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(0b1100, 0b1010, ctrl);

        // A: 1100
        // B: 1010
        // |: 1110
        assert_eq!(res.s, 0b1110);
        assert_eq!(res.sd, res.s);
    }

    #[test]
    fn not_b_operation() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: true,
            f1: false,
            ena: false,
            enb: true,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(0, 0b00001111, ctrl);
        // B: 00001111
        // !B: 11110000
        assert_eq!(res.s as u8, 0b11110000); // we need to make this a u8 cause of the left 1's :D
        assert_eq!(res.sd, res.s);
    }

    #[test]
    fn invert_a() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: false,
            f1: false,
            ena: true,
            enb: true,
            inva: true,
            inc: false,
        };

        //  A: 0000
        //  B: 1010
        // !A: 1111 (we need to invert it first, again :D)
        //  &: 1010
        let (_, res) = Alu::execute(0b0000, 0b1010, ctrl);
        assert_eq!(res.s, 0b1010);
        assert_eq!(res.sd, res.s);
    }

    #[test]
    fn disable_a() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: true,
            f1: true,
            ena: false,
            enb: true,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(34, 35, ctrl);
        // we should get the b again because a turns 0, not 69
        assert_eq!(res.s, 35);
        assert_eq!(res.sd, res.s);
    }

    #[test]
    fn complete_sum() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: true,
            f1: true,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(34, 35, ctrl);
        // now we should get the sum as expected
        assert_eq!(res.s, 69);
        assert_eq!(res.sd, res.s);
    }

    #[test]
    fn sum_with_disable_a_and_b() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: true,
            f1: true,
            ena: false,
            enb: false,
            inva: false,
            inc: false,
        };

        let (_, res) = Alu::execute(34, 35, ctrl);
        // as both operands are not enabled, we should get a zeroed result :/
        assert_eq!(res.s, 0);
        assert_eq!(res.sd, 0);
        assert!(res.z);
        assert!(!res.n);
    }

    #[test]
    fn sll8_after_add() {
        let ctrl = AluInstruction::from(0b10111100);
        let (_, res) = Alu::execute(0x00000001, 0x80000000, ctrl);
        assert_eq!(res.s, 0x80000001);
        assert_eq!(res.sd, 0x80000001u32.wrapping_shl(8));
        assert!(!res.n);
        assert!(!res.z);
    }

    #[test]
    fn sra1_after_add() {
        let ctrl = AluInstruction::from(0b01111100);
        let (_, res) = Alu::execute(0x00000001, 0x80000000, ctrl);
        assert_eq!(res.s, 0x80000001);
        let expected_sd = (res.s >> 1) | (res.s & 0x80000000);
        assert_eq!(res.sd, expected_sd);
        assert_eq!(res.sd, 0xC000_0000);
        assert!(res.n);
        assert!(!res.z);
    }

    #[test]
    fn invalid_both_shifts_sd_is_dont_care_but_s_computed() {
        let ctrl = AluInstruction::from(0b11111100);
        assert!(!ctrl.is_valid());
        let (_, res) = Alu::execute(0x00000001, 0x80000000, ctrl);
        assert_eq!(res.s, 0x80000001);
        assert_eq!(res.sd, 0);
        assert!(!res.n);
        assert!(!res.z);
    }

    #[test]
    fn from_str_parses_six_or_eight_bits() {
        let six: AluInstruction = "111110".parse().unwrap();
        let eight: AluInstruction = "00111110".parse().unwrap();
        assert_eq!(six, eight);
        assert_eq!(six, AluInstruction::from(0b00111110));
    }

    #[test]
    fn from_str_round_trips_display() {
        let ctrl = AluInstruction {
            sll8: false,
            sra1: false,
            f0: true,
            f1: false,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };
        let s = ctrl.to_string();
        assert_eq!(s.len(), 8);
        let parsed: AluInstruction = s.parse().unwrap();
        assert_eq!(parsed, ctrl);
    }

    #[test]
    fn from_str_rejects_wrong_length() {
        assert_eq!(
            "11111".parse::<AluInstruction>().unwrap_err(),
            AluParseError::InvalidLength(5)
        );
        assert_eq!(
            "1111111".parse::<AluInstruction>().unwrap_err(),
            AluParseError::InvalidLength(7)
        );
    }

    #[test]
    fn from_str_rejects_invalid_digits() {
        assert_eq!(
            "11111a".parse::<AluInstruction>().unwrap_err(),
            AluParseError::NonBinaryChar
        );
        assert_eq!(
            "0011110x".parse::<AluInstruction>().unwrap_err(),
            AluParseError::NonBinaryChar
        );
    }
}
