use crate::alu::{Alu, AluInstruction, AluResult, Inputs};

/// The CPU holds the state and orchestrates the execution of instructions.
pub struct Cpu {
    pc: usize,
}

/// The result of each executed program line.
/// That's all the information we need to be logged.
pub struct ExecutionLog {
    /// 8-bit instruction (`Display` / program lines may use 6 legacy bits)
    pub ir: AluInstruction,
    /// program counter
    pub pc: usize,
    /// `Alu` output
    pub result: AluResult,
    pub used_inputs: Inputs,
}

impl ExecutionLog {
    /// Creates a new `ExecutionLog`. Useful in tests.
    pub fn new(ir: AluInstruction, pc: usize, a: u32, b: u32, result: AluResult) -> Self {
        Self {
            ir,
            pc,
            result,
            used_inputs: Inputs { a, b },
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu { pc: 1 }
    }

    #[inline]
    /// Executes one instruction cycle.
    /// Returns the execution log for output handling.
    pub fn execute_cycle(&mut self, a: u32, b: u32, instruction: AluInstruction) -> ExecutionLog {
        // we need to get the current pc BEFORE the incrementing
        let pc = self.pc;

        // increments the pc for the next instruction :D
        self.pc += 1;
        let (used_inputs, result) = Alu::execute(a, b, instruction);

        ExecutionLog {
            ir: instruction,
            pc,
            result,
            used_inputs,
        }
    }
}
