use std::io::{Result, Write};

use crate::cpu::ExecutionLog;

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
        writeln!(self.writer, "============================================================")?;
        writeln!(self.writer, "Cycle {}", cycle)?;
        writeln!(self.writer)?;
        writeln!(self.writer, "PC     = {}", log.pc)?;
        writeln!(self.writer, "IR     = {}", log.ir)?;

        let (a, b) = match cycle > 1 {
            true => (log.used_inputs.a, log.used_inputs.b),
            _ => (log.a, log.b),
        };

        writeln!(self.writer, "b      = {:032b}", b)?;
        writeln!(self.writer, "a      = {:032b}", a)?;
        writeln!(self.writer, "s      = {:032b}", log.result.s())?;
        writeln!(self.writer, "co = {}", if log.result.carry() { 1 } else { 0 })?;

        Ok(())
    }

    /// Escreve a finalização do programa.
    pub fn end_program(&mut self, cycle: usize) -> Result<()> {
        writeln!(self.writer, "EOP")?;
        writeln!(self.writer, "Programa encerrado no ciclo {}", cycle)?;
        Ok(())
    }

    /// Retorna o writer interno.
    /// Útil em testes com `Vec<u8>` ou `Cursor<Vec<u8>>`.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alu::{AluInstruction, AluResult};
    use crate::cpu::ExecutionLog;

    #[test]
    fn logger_writes_to_memory() {
        let buffer = Vec::<u8>::new();
        let mut logger = Logger::new(buffer);

        let ir = AluInstruction::from(0b111100);
        let log = ExecutionLog::new(ir, 0, 10, 20, AluResult::new(30, false));

        logger.start_program(10, 20).unwrap();
        logger.log_cycle(1, &log).unwrap();
        logger.end_program(2).unwrap();

        let output = String::from_utf8(logger.into_inner()).unwrap();

        assert!(output.contains("INÍCIO DO PROGRAMA"));
        assert!(output.contains("Ciclo 1"));
        assert!(output.contains("IR     = 111100"));
        assert!(output.contains("S      = 30"));
        assert!(output.contains("Vai-um = 0"));
        assert!(output.contains("EOP"));
    }
}