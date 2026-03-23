mod alu;
mod cpu;
mod instruction_register;
mod loader;
mod logger;

use std::fs::File;
use std::io::BufWriter;

use cpu::Cpu;
use loader::load_program;
use logger::Logger;

fn main() -> std::io::Result<()> {
    let a: u32 = 0xFFFF_FFFF;
    let b: u32 = 1;

    let program = load_program("data/programa_etapa1.txt")
        .expect("Failed to load program");

    let file = File::create("data/saida_etapa1_generated.txt")?;
    let writer = BufWriter::new(file);

    let mut cpu = Cpu::new();
    let mut logger = Logger::new(writer);

    logger.start_program(a, b)?;

    for (i, ir_bits) in program.iter().copied().enumerate() {
        let log = cpu.execute_cycle(a, b, ir_bits);
        logger.log_cycle(i + 1, &log)?;
    }

    logger.end_program(program.len() + 1)?;
    Ok(())
}