use crate::cpu::ExecutionLog;
use crate::microinstruction::MicroInstruction;
use crate::register::Registers;
use std::io::{Result, Write};

/// Logger responsável por escrever a execução do programa em qualquer saída
/// compatível com `Write` (`File`, `BufWriter`, memória, etc.).
pub struct Logger<W: Write> {
    writer: W,
}

#[derive(Debug, PartialEq)]
/// Each
pub struct Cycle {}

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

    pub fn log_mic1_start(&mut self, regs: &Registers) -> std::io::Result<()> {
        writeln!(self.writer, "Start of Mic-1 Program")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "Initial Registers:")?;
        self.write_registers(regs)?;
        writeln!(self.writer)?;
        Ok(())
    }

    pub fn log_mic1_cycle(
        &mut self,
        cycle: usize,
        instr: &MicroInstruction,
        b_name: &str,
        c_names: &[&str],
        before: &Registers,
        after: &Registers,
    ) -> std::io::Result<()> {
        writeln!(self.writer, "============================================================")?;
        writeln!(self.writer, "Cycle {cycle}")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "IR = {instr}")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "B-bus: {b_name}")?;
        if c_names.is_empty() {
            writeln!(self.writer, "C-bus: (none)")?;
        } else {
            writeln!(self.writer, "C-bus: {}", c_names.join(", "))?;
        }
        writeln!(self.writer)?;
        writeln!(self.writer, "Registers (before):")?;
        self.write_registers(before)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "Registers (after):")?;
        self.write_registers(after)?;
        Ok(())
    }

    pub fn log_mic1_eop(&mut self, cycle: usize) -> std::io::Result<()> {
        writeln!(self.writer, "============================================================")?;
        writeln!(self.writer, "Cycle {cycle}")?;
        writeln!(self.writer)?;
        writeln!(self.writer, "> End of Program.")?;
        writeln!(self.writer)?;
        self.writer.flush()?;
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
}
