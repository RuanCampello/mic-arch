use crate::{
    alu::Alu,
    microinstruction::{c_bus_names, c_bus_write, MicroInstruction},
    register::Registers,
};

pub struct Mic1 {
    pub regs: Registers,
}

impl Mic1 {
    pub fn new(regs: Registers) -> Self {
        Self { regs }
    }

    pub fn step(
        &mut self,
        instr: &MicroInstruction,
    ) -> (&'static str, Vec<&'static str>, Registers, Registers) {
        let before = self.regs.clone();
        let b_name = Registers::b_bus_name(instr.b_sel);
        let b = self.regs.b_bus_decode(instr.b_sel);
        let a = self.regs.h;
        let (_, alu_result) = Alu::execute(a, b, instr.alu);
        let c_names = c_bus_names(instr.c_sel);
        c_bus_write(&mut self.regs, instr.c_sel, alu_result.sd);
        let after = self.regs.clone();
        (b_name, c_names, before, after)
    }
}
