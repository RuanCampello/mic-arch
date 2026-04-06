#![allow(unused)]

pub mod alu;
pub mod cpu;
pub mod instruction_register;
pub mod loader;
pub mod logger;

#[cfg(test)]
#[allow(unused)]
mod tests {
   use crate::{
        cpu::{self, Cpu},
        loader::load_program,
        logger::Logger,
    };
    use std::{fs::File, io::BufWriter, path::Path};

    #[test]
    fn output() -> Result<(), Box<dyn std::error::Error>> {
        let a: u32 = 0b00000000000000000000000000001111;
        let b: u32 = 0b00000000000000000000000011110000;

        let input = Path::new("./program.txt");
        let output = Path::new("./output.txt");

        let mut cpu = Cpu::new();
        let mut logger = Logger::new(File::create(&output)?);
        logger.start_program(a, b)?;

        let program = load_program(&input)?;
        for (cycle, &instruction) in program.iter().enumerate() {
            let result = cpu.execute_cycle(a, b, instruction);
            logger.log_cycle(cycle + 1, &result)?;
        }

        logger.end_program(program.len() + 1)?;
        println!("{}", std::fs::read_to_string(output)?);

        Ok(())
    }
}