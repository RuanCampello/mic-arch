pub mod alu;
pub mod cpu;
pub mod instruction_register;
pub mod loader;
pub mod logger;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{cpu::Cpu, loader::load_program, logger::Logger};
    use std::{fs::File, io::BufWriter, path::Path};

    #[test]
    fn output() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
