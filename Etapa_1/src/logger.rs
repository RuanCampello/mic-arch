use crate::cpu::ExecutionLog;
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
            self.writer, "============================================================"
        )?;
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {}", log.pc)?;
        writeln!(self.writer, "IR = {}", log.ir)?;
        writeln!(self.writer, "b = {:032b}", log.used_inputs.b)?;
        writeln!(self.writer, "a = {:032b}", log.used_inputs.a)?;
        writeln!(self.writer, "s = {:032b}", log.result.s)?;
        writeln!(self.writer, "co = {}", if log.result.carry { 1 } else { 0 })?;
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
        write!(self.writer, "> Line is empty, EOP.")?;

        self.writer.flush()?;
        Ok(())
    }
}
