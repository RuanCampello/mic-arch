pub mod alu;
pub mod cpu;
pub mod instruction_register;
pub mod loader;
pub mod logger;
pub mod mic1;
pub mod microinstruction;
pub mod register;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{
        alu::AluInstruction,
        cpu::{self, Cpu},
        loader::load_program,
        logger::Logger,
    };
    use std::{fs::File, io::BufWriter, path::Path, str::FromStr};

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
            if instruction.is_valid() {
                logger.log_cycle(cycle + 1, &result)?;
            } else {
                logger.log_cycle_invalid_signals(cycle + 1, &result)?;
            }
        }

        logger.end_program(program.len() + 1)?;
        println!("{}", std::fs::read_to_string(output)?);

        Ok(())
    }

    #[test]
    fn etapa_2_output() -> Result<(), Box<dyn std::error::Error>> {
        let a: u32 = 0x000000FF;
        let b: u32 = 0x0000FF00;

        //   00001100 — AND(a,b):    s=0x00000000, z=1
        //   01100100 — SRA1+NOT_B:  s=0xFFFF00FF, sd=0xFFFF807F, n=1
        //   10111100 — SLL8+ADD:    s=0x0000FFFF, sd=0x00FFFF00

        let program: Vec<AluInstruction> = ["00001100", "01100100", "10111100"]
            .iter()
            .filter_map(|s| AluInstruction::from_str(s).ok())
            .collect();

        let mut cpu = Cpu::new();
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut logger = Logger::new(&mut buf);
        logger.start_program(a, b)?;

        // ciclo 1 — AND: resultado zerado, z=1
        let r0 = cpu.execute_cycle(a, b, program[0]);
        assert!(program[0].is_valid());
        assert_eq!(r0.result.s, 0x0000_0000);
        assert_eq!(r0.result.sd, 0x0000_0000);
        assert!(r0.result.z);
        assert!(!r0.result.n);
        assert!(!r0.result.carry);
        logger.log_cycle(1, &r0)?;

        // ciclo 2 — NOT_B + SRA1: bit 31 de sd setado, n=1
        let r1 = cpu.execute_cycle(a, b, program[1]);
        assert!(program[1].is_valid());
        assert_eq!(r1.result.s, 0xFFFF_00FF);
        assert_eq!(r1.result.sd, 0xFFFF_807F);
        assert!(r1.result.n);
        assert!(!r1.result.z);
        logger.log_cycle(2, &r1)?;

        // ciclo 3 — ADD + SLL8: sem carry, sd deslocado 8 bits à esquerda
        let r2 = cpu.execute_cycle(a, b, program[2]);
        assert!(program[2].is_valid());
        assert_eq!(r2.result.s, 0x0000_FFFF);
        assert_eq!(r2.result.sd, r2.result.s.wrapping_shl(8));
        assert_eq!(r2.result.sd, 0x00FF_FF00);
        assert!(!r2.result.n);
        assert!(!r2.result.z);
        assert!(!r2.result.carry);
        logger.log_cycle(3, &r2)?;

        logger.end_program(program.len() + 1)?;

        let output = String::from_utf8(buf.into_inner())?;
        println!("{output}");

        Ok(())
    }
}
