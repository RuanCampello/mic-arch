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
    pub ir: AluInstruction,
    /// program counter
    pub pc: usize,
    /// `Alu` input A effectively used by the ALU
    pub a: u32,
    /// `Alu` input B effectively used by the ALU
    pub b: u32,
    /// `Alu` output
    pub result: AluResult,
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
    pub fn execute_cycle(
        &mut self,
        a: u32,
        b: u32,
        instruction: AluInstruction,
    ) -> ExecutionLog {
        // loads the instruction into IR
        self.ir.load(instruction);

        // we need to get the current pc BEFORE the incrementing
        let pc = self.pc.get();

        let result = Alu::execute(a, b, instruction);

        // values actually considered by the ALU after enable signals
        let logged_a = if instruction.ena() { a } else { 0 };
        let logged_b = if instruction.enb() { b } else { 0 };

        // increments the pc for the next instruction :D
        self.pc.increment();

        ExecutionLog {
            ir: instruction,
            pc,
            a: logged_a,
            b: logged_b,
            result,
        }
    }
}