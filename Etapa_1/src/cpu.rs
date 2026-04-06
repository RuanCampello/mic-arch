use crate::{
    alu::{Alu, AluInstruction, AluResult, Inputs},
    instruction_register::{InstructionRegister, ProgramCounter},
};

/// The CPU holds the state and orchestrates the execution of instructions.
pub struct Cpu {
    pc: ProgramCounter,
    ir: InstructionRegister,
}

/// The result of each executed program line.
/// That's all the information we need to be logged.
pub struct ExecutionLog {
    /// 6-bit instruction
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
        Cpu {
            pc: ProgramCounter::new(),
            ir: InstructionRegister::new(),
        }
    }

    #[inline]
    /// Executes one instruction cycle.
    /// Returns the execution log for output handling.
    pub fn execute_cycle(&mut self, a: u32, b: u32, instruction: AluInstruction) -> ExecutionLog {
        // loads the instruction into IR
        self.ir.load(instruction);

        // we need to get the current pc BEFORE the incrementing
        let pc = self.pc.get();
        let (used_inputs, result) = Alu::execute(a, b, instruction);

        // increments the pc for the next instruction :D
        self.pc.increment();

        ExecutionLog {
            ir: instruction,
            pc,
            result,
            used_inputs
        }
    }
}