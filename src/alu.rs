//! Arithmetic logic unit `mic-1` implementation.

/// The arithmetic logic unit of `mic-1`.
/// This performs arithmetic and logic (duh) operations on `A` and `B`
/// according to the [control](self::AluControl) signals.
///
/// Although this keeps no state, we use for scope purposes.
pub(crate) struct Alu;

/// The `ALU` returns both the result and the carry (vai-um).
pub(crate) struct AluResult {
    s: u32,
    carry: bool,
}

/// The `ALU` receives control instructions with the following 6-bit format:
///
/// | Bit | Signal | Description |
/// |-----|--------|-------------|
/// | 5   | F0     | selects ALU operation |
/// | 4   | F1     | selects ALU operation |
/// | 3   | ENA    | enables input `a`. if `0`, `a` is treated as `0`. |
/// | 2   | ENB    | enables input `b`. if `0`, `b` is treated as `0`. |
/// | 1   | INVA   | inverts input `a` before the ALU operation. |
/// | 0   | INC    | forces carry-in = 1, effectively adding `+1` to the result. |
///
/// The `F0` and `F1` bits determine the core ALU operation:
///
/// | F1 | F0 | Operation |
/// |----|----|-----------|
/// | 0  | 0  | `A & B` (logic AND)  |
/// | 0  | 1  | `A \| B` (logic OR) |
/// | 1  | 0  | `!B` (logic NOT) |
/// | 1  | 1  | `A + B` (arithmetic ADD)|
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AluControl {
    f0: bool,
    f1: bool,
    ena: bool,
    enb: bool,
    inva: bool,
    inc: bool,
}

impl Alu {
    pub fn execute(a: u32, b: u32, control: AluControl) -> AluResult {
        // verify which inputs are enabled
        let mut a = if control.ena { a } else { 0 };
        let b = if control.enb { b } else { 0 };

        // verify if we need to invert A BEFORE the operation
        if control.inva {
            a = !a;
        }

        let mut carry = false;

        // this follows the spec interpretation of bits for f1, f0.
        let mut result = match (control.f1, control.f0) {
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
        // here we need to take a little care of inc it because it can overflow too
        // in that case, we also need to flip the carry
        if control.inc {
            let (s, c) = result.overflowing_add(1);
            result = s;
            carry |= c;
        }

        AluResult { s: result, carry }
    }
}

impl From<u8> for AluControl {
    fn from(bits: u8) -> Self {
        Self {
            f0: bits & 0b100000 != 0,
            f1: bits & 0b010000 != 0,
            ena: bits & 0b001000 != 0,
            enb: bits & 0b000100 != 0,
            inva: bits & 0b000010 != 0,
            inc: bits & 0b000001 != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_operation() {
        let ctrl = AluControl {
            f0: false,
            f1: false,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(0b1100, 0b1010, ctrl);

        // A: 1101
        // B: 1010
        // &: 1000
        assert_eq!(res.s, 0b1000);
        assert!(!res.carry);
    }

    #[test]
    fn or_operation() {
        let ctrl = AluControl {
            f0: true,
            f1: false,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(0b1100, 0b1010, ctrl);

        // A: 1100
        // B: 1010
        // |: 1110
        assert_eq!(res.s, 0b1110);
    }

    #[test]
    fn not_b_operation() {
        let ctrl = AluControl {
            f0: false,
            f1: true,
            ena: false,
            enb: true,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(0, 0b00001111, ctrl);
        // B: 00001111
        // !B: 11110000
        assert_eq!(res.s as u8, 0b11110000); // we need to make this a u8 cause of the left 1's :D
    }

    #[test]
    fn invert_a() {
        let ctrl = AluControl {
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
        let res = Alu::execute(0b0000, 0b1010, ctrl);
        assert_eq!(res.s, 0b1010);
    }

    #[test]
    fn disable_a() {
        let ctrl = AluControl {
            f0: true,
            f1: true,
            ena: false,
            enb: true,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(34, 35, ctrl);
        // we should get the b again because a turns 0, not 69
        assert_eq!(res.s, 35);
    }

    #[test]
    fn complete_sum() {
        let ctrl = AluControl {
            f0: true,
            f1: true,
            ena: true,
            enb: true,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(34, 35, ctrl);
        // now we should get the sum as expected
        assert_eq!(res.s, 69);
    }

    #[test]
    fn sum_with_disable_a_and_b() {
        let ctrl = AluControl {
            f0: true,
            f1: true,
            ena: false,
            enb: false,
            inva: false,
            inc: false,
        };

        let res = Alu::execute(34, 35, ctrl);
        // as both operands are not enabled, we should get a zeroed result :/
        assert_eq!(res.s, 0);
    }
}
