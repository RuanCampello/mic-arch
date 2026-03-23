use std::io::{Result, Write};

use crate::cpu::ExecutionLog;

pub struct Logger<W: Write> {
    writer: W,
}

impl<W: Write> Logger<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Writes the initial values and program header.
    pub fn start_program(&mut self, a: u32, b: u32) -> Result<()> {
        writeln!(self.writer, "b = {:032b}", b)?;
        writeln!(self.writer, "a = {:032b}", a)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "Start of Program")?;
        writeln!(self.writer, "============================================================")?;
        Ok(())
    }

    /// Writes one execution cycle in the expected Etapa 1 format.
    pub fn log_cycle(&mut self, cycle: usize, log: &ExecutionLog) -> Result<()> {
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {}", log.pc)?;
        writeln!(self.writer, "IR = {}", log.ir)?;
        writeln!(self.writer, "b = {:032b}", log.b)?;
        writeln!(self.writer, "a = {:032b}", log.a)?;
        writeln!(self.writer, "s = {:032b}", log.result.s())?;
        writeln!(self.writer, "co = {}", if log.result.carry() { 1 } else { 0 })?;
        writeln!(self.writer, "============================================================")?;
        Ok(())
    }

    /// Writes the final EOP marker.
    pub fn end_program(&mut self, cycle: usize) -> Result<()> {
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC = {}", cycle)?;
        writeln!(self.writer, "> Line is empty, EOP.")?;
        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}