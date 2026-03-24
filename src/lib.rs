#![allow(unused)]

mod alu;
mod cpu;
mod instruction_register;
mod loader;
mod logger;

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufWriter};
    use crate::{cpu::Cpu, loader::load_program, logger::Logger};

    #[test]
    fn output() -> Result<(), Box<dyn std::error::Error>> {
        let b: u32 = 0b00000000000000000000000000000001;
        let a: u32 = 0b11111111111111111111111111111111;

        let file = File::create("./output/saida_etapa1_output.txt")?;
        let writer = BufWriter::new(file);

        let mut cpu = Cpu::new();
        let mut logger = Logger::new(writer);
        logger.start_program(a, b)?;

        let program = load_program("./expected/programa_etapa1.txt")?;
        for (cycle, &instruction) in program.iter().enumerate() {
            let result = cpu.execute_cycle(a, b, instruction);
            logger.log_cycle(cycle + 1, &result)?;
        }

        logger.end_program(program.len() + 1);

        Ok(())
    }
}
