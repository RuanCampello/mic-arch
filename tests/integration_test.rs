use mic_arch::{cpu::Cpu, loader::load_program, logger::Logger};
use std::io::Cursor;
use std::path::Path;

/// Etapa 1 reference program (6 bits/line): results must match the ALU spec — no `.txt` golden comparison.
#[test]
fn test_etapa1_program_execution_matches_spec() -> Result<(), Box<dyn std::error::Error>> {
    let b: u32 = 0b00000000000000000000000000000001;
    let a: u32 = 0b11111111111111111111111111111111;

    let path = Path::new("./tests/data/programa_etapa1.txt");
    let program = load_program(path)?;
    assert_eq!(program.len(), 4);

    let mut cpu = Cpu::new();

    let r0 = cpu.execute_cycle(a, b, program[0]);
    assert!(program[0].is_valid());
    assert_eq!(r0.result.s, 0);
    assert!(r0.result.carry);
    assert_eq!(r0.result.sd, 0);
    assert!(r0.result.z);
    assert!(!r0.result.n);

    let r1 = cpu.execute_cycle(a, b, program[1]);
    assert!(program[1].is_valid());
    assert_eq!(r1.result.s, 2);
    assert_eq!(r1.result.sd, 2);
    assert!(!r1.result.z);

    let r2 = cpu.execute_cycle(a, b, program[2]);
    assert!(program[2].is_valid());
    assert_eq!(r2.result.s, 1);
    assert_eq!(r2.result.sd, 1);

    let r3 = cpu.execute_cycle(a, b, program[3]);
    assert!(program[3].is_valid());
    assert_eq!(r3.result.s, 0xFFFF_FFFF);
    assert_eq!(r3.result.sd, 0xFFFF_FFFF);
    assert!(r3.result.n);
    assert!(!r3.result.z);

    Ok(())
}

/// Etapa 2 reference program: 8-bit words including an invalid line.
#[test]
fn test_etapa2_program_execution_matches_spec() -> Result<(), Box<dyn std::error::Error>> {
    let b: u32 = 0x80000000;
    let a: u32 = 0x00000001;

    let path = Path::new("./tests/data/programa_etapa2_tarefa1.txt");
    let program = load_program(path)?;
    assert_eq!(program.len(), 3);

    let mut cpu = Cpu::new();

    let r0 = cpu.execute_cycle(a, b, program[0]);
    assert!(program[0].is_valid());
    assert_eq!(r0.result.s, 0x80000001);
    assert_eq!(r0.result.sd, 0x80000001u32.wrapping_shl(8));
    assert_eq!(r0.pc, 1);

    let r1 = cpu.execute_cycle(a, b, program[1]);
    assert!(program[1].is_valid());
    assert_eq!(r1.result.s, 0x80000001);
    assert_eq!(
        r1.result.sd,
        (r1.result.s >> 1) | (r1.result.s & 0x80000000)
    );
    // n = (sd >> 31) == 1 — sd = 0xC0000000 → n must be set
    assert!(r1.result.n);
    assert!(!r1.result.z);
    assert_eq!(r1.pc, 2);

    let r2 = cpu.execute_cycle(a, b, program[2]);
    assert!(!program[2].is_valid());
    assert_eq!(r2.result.s, 0x80000001);
    assert_eq!(r2.pc, 3);

    Ok(())
}

/// Logger para o programa da etapa 2: checa formato e valores coerentes com a ALU (n/z a partir de `sd`).
/// Não lê `tests/data/*.txt` de especificação — só o programa binário é carregado.
#[test]
fn test_etapa2_logger_output_matches_execution() -> Result<(), Box<dyn std::error::Error>> {
    let a: u32 = 0x00000001;
    let b: u32 = 0x80000000;
    let path = Path::new("./tests/data/programa_etapa2_tarefa1.txt");
    let program = load_program(path)?;
    assert_eq!(program.len(), 3);

    let mut buf = Cursor::new(Vec::new());
    let mut logger = Logger::new(&mut buf);
    logger.start_program(a, b)?;

    let mut cpu = Cpu::new();
    for (cycle, &instruction) in program.iter().enumerate() {
        let log = cpu.execute_cycle(a, b, instruction);
        if instruction.is_valid() {
            logger.log_cycle(cycle + 1, &log)?;
        } else {
            logger.log_cycle_invalid_signals(cycle + 1, &log)?;
        }
    }
    logger.end_program(program.len() + 1)?;

    let s = String::from_utf8(buf.into_inner())?.replace("\r\n", "\n");
    assert!(s.contains("Start of Program"));
    assert!(s.contains("IR = 10111100"));
    assert!(s.contains("IR = 01111100"));
    assert!(s.contains("IR = 11111100"));
    assert!(s.contains("sd = 00000000000000000000000100000000")); // sll8: 0x80000001<<8 em u32
    assert!(s.contains("sd = 11000000000000000000000000000000"));
    assert!(s.contains("n = 1")); // n = (sd>>31) após SRA1 → 0xC0000000
    assert!(s.contains("z = 0"));
    assert!(s.contains("co = 0"));
    assert!(s.contains("> Error, invalid control signals."));
    assert!(s.contains("> Line is empty, EOP."));
    assert!(!s.contains("IR = 11111100\nsd =")); // inválido: sem `sd`/`n`/`z`/`co` após o IR

    Ok(())
}

#[test]
fn test_logger_invalid_cycle_format() -> Result<(), Box<dyn std::error::Error>> {
    use mic_arch::alu::{AluInstruction, AluResult, Inputs};
    use mic_arch::cpu::ExecutionLog;

    let mut buf = Cursor::new(Vec::new());
    let mut logger = Logger::new(&mut buf);

    let ir: AluInstruction = "11111100".parse().unwrap();
    let log = ExecutionLog {
        ir,
        pc: 3,
        result: AluResult {
            s: 0,
            sd: 0,
            carry: false,
            n: false,
            z: false,
        },
        used_inputs: Inputs { a: 1, b: 2 },
    };

    logger.log_cycle_invalid_signals(3, &log)?;
    let s = String::from_utf8(buf.into_inner())?;
    assert!(s.contains("PC = 3"));
    assert!(s.contains("IR = 11111100"));
    assert!(s.contains("> Error, invalid control signals."));
    assert!(!s.contains("sd ="));

    Ok(())
}
