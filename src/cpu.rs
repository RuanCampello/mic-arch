use crate::{
    alu::{Alu, AluInstruction, AluResult},
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
    ir: u8,
    /// program counter
    pc: usize,
    /// `Alu` input A
    a: u32,
    /// `Alu` input A
    b: u32,
    /// `Alu` output
    result: AluResult,
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
    pub fn execute_cycle(&mut self, a: u32, b: u32, instruction: u8) -> ExecutionLog {
        // loads the instruction into IR
        let alu_instruction = AluInstruction::from(instruction);
        self.ir.load(alu_instruction);

        // we need to get the current pc BEFORE the incrementing
        let pc = self.pc.get();
        let result = Alu::execute(a, b, alu_instruction);

        // increments the pc for the next instruction :D
        self.pc.increment();

        ExecutionLog {
            ir: instruction,
            pc,
            a,
            b,
            result,
        }
    }
}
