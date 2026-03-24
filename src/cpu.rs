use crate::{
    alu::{Alu, AluInstruction, AluResult, Inputs},
    instruction_register::{InstructionRegister, ProgramCounter},
};

/// The CPU holds the state and orchestrates the execution of instructions.
pub struct Cpu {
    pc: ProgramCounter,
    ir: InstructionRegister,
    alu: Alu,
}

/// The result of each executed program line.
/// That's all the information we need to be logged.
pub struct ExecutionLog {
    /// 6-bit instruction
    pub ir: AluInstruction,
    /// program counter
    pub pc: usize,
    /// `Alu` input A
    pub a: u32,
    /// `Alu` input A
    pub b: u32,
    /// `Alu` output
    pub result: AluResult,
    pub used_inputs: Inputs,
}

impl ExecutionLog {
    /// Creates a new `ExecutionLog`. Useful in tests.
    pub fn new(
        ir: AluInstruction,
        pc: usize,
        a: u32,
        b: u32,
        result: AluResult,
    ) -> Self {
        Self {
            ir,
            pc,
            a,
            b,
            result,
            used_inputs: Inputs {a, b},
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: ProgramCounter::new(),
            ir: InstructionRegister::new(),
            alu: Alu,
        }
    }

    #[inline]
    /// Executes one instruction cycle.
    /// Returns the execution log for output handling.
    pub fn execute_cycle(&mut self, a: u32, b: u32, instruction: AluInstruction) -> ExecutionLog {
        self.ir.load(instruction);

        // we need to get the current pc BEFORE the incrementing
        let pc = self.pc.get();
        let (used_inputs, result) = Alu::execute(a, b, instruction);

        // increments the pc for the next instruction :D
        self.pc.increment();

        ExecutionLog {
            ir: instruction,
            pc,
            a,
            b,
            result,
            used_inputs,
        }
    }
}
