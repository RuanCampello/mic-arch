use crate::microinstruction::MicroInstruction;
use crate::register::Registers;
use crate::{cpu::ExecutionLog, memory::Memory};
use std::io::{Result, Write};

/// Logger responsável por escrever a execução do programa em qualquer saída
/// compatível com `Write` (`File`, `BufWriter`, memória, etc.).
pub struct Logger<W: Write> {
    writer: W,
}

impl<W: Write> Logger<W> {
    /// Cria um novo logger usando o writer fornecido.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Escreve o cabeçalho inicial do programa.
    pub fn start_program(&mut self, a: u32, b: u32) -> Result<()> {
        writeln!(self.writer, "b = {:032b}", b)?;
        writeln!(self.writer, "a = {:032b}", a)?;
        writeln!(self.writer)?;

        writeln!(self.writer, "Start of Program")?;
        Ok(())
    }

    /// Escreve o log de um ciclo/instrução.
    pub fn log_cycle(&mut self, cycle: usize, log: &ExecutionLog) -> Result<()> {
        writeln!(
            self.writer,
            "============================================================"
        )?;
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {}", log.pc)?;
        writeln!(self.writer, "IR = {}", log.ir)?;
        writeln!(self.writer, "b = {:032b}", log.used_inputs.b)?;
        writeln!(self.writer, "a = {:032b}", log.used_inputs.a)?;
        writeln!(self.writer, "s = {:032b}", log.result.s)?;
        writeln!(self.writer, "sd = {:032b}", log.result.sd)?;
        writeln!(self.writer, "n = {}", if log.result.n { 1 } else { 0 })?;
        writeln!(self.writer, "z = {}", if log.result.z { 1 } else { 0 })?;
        writeln!(self.writer, "co = {}", if log.result.carry { 1 } else { 0 })?;

        Ok(())
    }

    /// Ciclo com `SLL8` e `SRA1` simultâneos: cabeçalho até `IR`, depois erro.
    pub fn log_cycle_invalid_signals(&mut self, cycle: usize, log: &ExecutionLog) -> Result<()> {
        writeln!(
            self.writer,
            "============================================================"
        )?;
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {}", log.pc)?;
        writeln!(self.writer, "IR = {}", log.ir)?;
        self.log_invalid()
    }

    /// Mensagem de erro para combinação inválida de sinais de deslocamento (`SLL8` e `SRA1`).
    pub fn log_invalid(&mut self) -> Result<()> {
        writeln!(self.writer, "> Error, invalid control signals.")?;
        Ok(())
    }

    fn write_registers(&mut self, regs: &Registers) -> std::io::Result<()> {
        writeln!(self.writer, "  H   = {:032b}", regs.h)?;
        writeln!(self.writer, "  OPC = {:032b}", regs.opc)?;
        writeln!(self.writer, "  TOS = {:032b}", regs.tos)?;
        writeln!(self.writer, "  CPP = {:032b}", regs.cpp)?;
        writeln!(self.writer, "  LV  = {:032b}", regs.lv)?;
        writeln!(self.writer, "  SP  = {:032b}", regs.sp)?;
        writeln!(self.writer, "  MBR = {:08b}", regs.mbr)?;
        writeln!(self.writer, "  PC  = {:032b}", regs.pc)?;
        writeln!(self.writer, "  MDR = {:032b}", regs.mdr)?;
        writeln!(self.writer, "  MAR = {:032b}", regs.mar)?;
        Ok(())
    }

    /// Escreve a finalização do programa.
    pub fn end_program(&mut self, cycle: usize) -> Result<()> {
        writeln!(
            self.writer,
            "============================================================"
        )?;
        writeln!(self.writer, "Cycle {cycle}")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {cycle}")?;
        writeln!(self.writer, "> Line is empty, EOP.")?;
        writeln!(self.writer)?;

        self.writer.flush()?;

        Ok(())
    }

    pub fn log_ijvm_cycle(
        &mut self,
        cycle: usize,
        instr: &MicroInstruction,
        b_name: &str,
        c_names: &[&str],
        before: &Registers,
        after: &Registers,
    ) -> Result<()> {
        writeln!(self.writer, "Cycle {cycle}")?;
        writeln!(self.writer, "ir = {instr}")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "b = {b_name}")?;

        match c_names.is_empty() {
            true => writeln!(self.writer, "c = (none)")?,
            _ => writeln!(self.writer, "c = {}", c_names.join(", "))?,
        };

        writeln!(self.writer)?;
        writeln!(self.writer, "> Registers before instruction")?;
        writeln!(self.writer, "*******************************")?;
        self.write_registers(before)?;

        writeln!(self.writer)?;
        writeln!(self.writer, "> Registers after instruction")?;
        writeln!(self.writer, "*******************************")?;
        self.write_registers(after)?;

        Ok(())
    }

    pub fn log_memory_after_instruction(&mut self, mem: &Memory) -> Result<()> {
        writeln!(self.writer)?;
        writeln!(self.writer, "> Memory after instruction")?;
        writeln!(self.writer, "*******************************")?;

        self.write_memory(mem)?;

        writeln!(
            self.writer,
            "============================================================"
        )?;
        Ok(())
    }

    pub fn log_ijvm_eop(&mut self, cycle: usize) -> Result<()> {
        writeln!(self.writer, "Cycle {cycle}")?;
        writeln!(self.writer, "No more lines, EOP.")?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn log_ijvm_start(&mut self) -> Result<()> {
        writeln!(self.writer, "Start of Program")?;
        writeln!(
            self.writer,
            "============================================================"
        )?;
        Ok(())
    }

    pub fn log_ijvm_initial_state(&mut self, mem: &Memory, regs: &Registers) -> Result<()> {
        writeln!(
            self.writer,
            "============================================================"
        )?;
        writeln!(self.writer, "Initial memory state")?;
        writeln!(self.writer, "*******************************")?;
        self.write_memory(&mem)?;
        writeln!(self.writer, "*******************************")?;
        writeln!(self.writer, "Initial register state")?;
        writeln!(self.writer, "*******************************")?;
        self.write_registers(regs)?;
        writeln!(
            self.writer,
            "============================================================"
        )?;
        Ok(())
    }

    pub fn write_memory(&mut self, mem: &Memory) -> Result<()> {
        let lines = mem
            .0
            .iter()
            .map(|v| format!("{:032b}", v))
            .collect::<Vec<_>>();

        for line in lines {
            writeln!(self.writer, "{line}")?;
        }

        Ok(())
    }
}
