pub mod alu;
pub mod cpu;
pub mod instruction_register;
pub mod loader;
pub mod logger;
pub mod memory;
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
        memory::Memory,
        microinstruction::MicroInstruction,
        register::Registers,
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

    #[test]
    fn etapa_3_output() -> Result<(), Box<dyn std::error::Error>> {
        // Instr 1: 00110101 000010000 01 0101
        //   ALU: ADD, ena=0, enb=1, inc=1 → 0 + lv + 1   c=lv   MEM=read
        // Instr 2: 00110100 000000100 10 0100
        //   ALU: ADD, ena=0, enb=1, inc=0 → 0 + sp        c=pc   MEM=write

        let i1: MicroInstruction = "00110101000010000010101".parse()?;
        let i2: MicroInstruction = "00110100000000100100100".parse()?;

        // lv=3, sp=5, mar=1; memory[1]=42
        let mut regs = Registers::default();
        regs.lv = 3;
        regs.sp = 5;
        regs.mar = 1;
        let mut mem = Memory::zero();
        mem.0[1] = 42;

        // Ciclo 1: lv = 0+3+1 = 4; mdr = memory[mar=1] = 42
        let (r1, m1) = i1.execute_micro_cycle(&regs, &mem);
        assert_eq!(r1.lv, 4);
        assert_eq!(r1.mdr, 42);
        assert_eq!(m1, mem); // sem escrita neste ciclo

        // Ciclo 2: pc = 0+sp=5; memory[mar=1] = mdr=42 (mar não muda, c=pc)
        let (r2, m2) = i2.execute_micro_cycle(&r1, &m1);
        assert_eq!(r2.pc, 5);
        assert_eq!(m2.0[1], 42);

        Ok(())
    }
}
