use std::{io::Write, path::Path, str::FromStr};

use crate::{
    alu::AluInstruction,
    logger::Logger,
    memory::{BBus, Memory, MemoryOperation},
    microinstruction::{MicroInstruction, c_bus_names},
    register::{RegisterParseError, Registers},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IjvmInstruction {
    Bipush(u8),
    Dup,
    ILoad(u8),
}

#[derive(Debug)]
pub enum IjvmParseError {
    Io(std::io::Error),
    InvalidLine { line: usize, message: String },
}

#[derive(Debug)]
pub enum ExecuteError {
    Ijvm(IjvmParseError),
    Registers(RegisterParseError),
    Io(std::io::Error),
}

impl IjvmInstruction {
    pub fn load(path: impl AsRef<Path>) -> Result<Vec<IjvmInstruction>, IjvmParseError> {
        let contents = std::fs::read_to_string(path).map_err(|io| IjvmParseError::Io(io))?;
        let mut program = Vec::new();

        for (idx, line) in contents.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let instruction =
                IjvmInstruction::from_str(line).map_err(|msg| IjvmParseError::InvalidLine {
                    line: idx,
                    message: msg,
                })?;
            program.push(instruction)
        }

        Ok(program)
    }
}

pub fn execute<W: Write>(
    instructions: impl AsRef<Path>,
    registers: impl AsRef<Path>,
    memory: impl AsRef<Path>,
    writer: W,
) -> Result<(), ExecuteError> {
    let ijvm = IjvmInstruction::load(instructions)?;
    let mut regs = Registers::load(registers)?;
    let mut mem = Memory::load(memory)?;

    let mut logger = Logger::new(writer);
    let mut cycle = 0;

    logger.log_ijvm_initial_state(&mem, &regs)?;
    logger.log_ijvm_start()?;

    for instruction in ijvm {
        let micro_instructions = translate(instruction);

        for micro in micro_instructions {
            let before = regs;
            let (after, new_memory) = micro.execute(&regs, &mem);

            let b_name = match micro.memory == MemoryOperation::ReadWrite {
                true => "(none)",
                _ => Registers::b_bus_name(micro.b_sel as u8),
            };

            let c_names = match micro.memory == MemoryOperation::ReadWrite {
                true => Vec::new(),
                _ => c_bus_names(micro.c_sel),
            };

            logger.log_ijvm_cycle(cycle, &micro, b_name, &c_names, &before, &after)?;

            regs = after;
            mem = new_memory;
            cycle += 1;
        }

        logger.log_memory_after_instruction(&mem)?;
    }
    logger.log_ijvm_eop(cycle)?;

    Ok(())
}

fn translate(instr: IjvmInstruction) -> Vec<MicroInstruction> {
    const PASS_A: u8 = 0b0011_1000;
    const PASS_B: u8 = 0b0011_0100;
    const A_PLUS_1: u8 = 0b0011_1001;
    const B_PLUS_1: u8 = 0b0011_0101;

    #[inline]
    fn micro(alu: u8, c_sel: u16, memory: MemoryOperation, b_sel: BBus) -> MicroInstruction {
        MicroInstruction {
            memory,
            alu: AluInstruction::from(alu),
            c_sel,
            b_sel,
        }
    }

    #[inline]
    fn fetch(byte: u8) -> MicroInstruction {
        MicroInstruction {
            alu: AluInstruction::from(byte),
            c_sel: 0,
            b_sel: BBus::Mdr,
            memory: MemoryOperation::ReadWrite,
        }
    }

    #[inline]
    fn mask(bits: &[u8]) -> u16 {
        let mut mask = 0;

        for &bit in bits {
            mask |= 1 << bit;
        }

        mask
    }

    match instr {
        IjvmInstruction::Bipush(byte) => vec![
            // SP = MAR = SP + 1
            micro(
                B_PLUS_1,
                mask(&[3, 0]), // SP, MAR
                MemoryOperation::None,
                BBus::Sp,
            ),
            // special fetch: MBR = byte; H = MBR (zero-extend)
            fetch(byte),
            // MDR = TOS = H; wr
            micro(
                PASS_A,
                mask(&[6, 1]), // TOS, MDR
                MemoryOperation::Write,
                BBus::Mdr,
            ),
        ],

        IjvmInstruction::Dup => vec![
            // SP = MAR = SP + 1
            micro(
                B_PLUS_1,
                mask(&[3, 0]), // SP, MAR
                MemoryOperation::None,
                BBus::Sp,
            ),
            // MDR = TOS; wr
            micro(
                PASS_B,
                mask(&[1]), // MDR
                MemoryOperation::Write,
                BBus::Tos,
            ),
        ],

        IjvmInstruction::ILoad(x) => {
            let mut out = Vec::new();
            // H = LV
            out.push(micro(
                PASS_B,
                mask(&[8]), // H
                MemoryOperation::None,
                BBus::Lv,
            ));
            // H = H + 1 (x vezes)
            for _ in 0..x {
                out.push(micro(
                    A_PLUS_1,
                    mask(&[8]), // H
                    MemoryOperation::None,
                    BBus::Mdr,
                ));
            }

            // MAR = H; rd
            out.push(micro(
                PASS_A,
                mask(&[0]), // MAR
                MemoryOperation::Read,
                BBus::Mdr,
            ));

            // MAR = SP = SP+1; wr
            out.push(micro(
                B_PLUS_1,
                mask(&[3, 0]), // SP, MAR
                MemoryOperation::Write,
                BBus::Sp,
            ));

            // TOS = MDR
            out.push(micro(
                PASS_B,
                mask(&[6]), // TOS
                MemoryOperation::None,
                BBus::Mdr,
            ));

            out
        }
    }
}

impl FromStr for IjvmInstruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err("empty line".into());
        }

        let mut parts = raw.split_whitespace();
        let op = parts.next().ok_or("missing opcode")?;

        match op.to_uppercase().as_str() {
            "BIPUSH" => {
                let arg = parts
                    .next()
                    .ok_or("BIPUSH requires 8-bit binary argument")?;
                if arg.len() != 8 || !arg.chars().all(|c| c == '0' || c == '1') {
                    return Err("BIPUSH argument must be 8-bit binary".into());
                }

                let byte = u8::from_str_radix(arg, 2).map_err(|_| "invalid binary")?;

                Ok(IjvmInstruction::Bipush(byte))
            }

            "DUP" => Ok(IjvmInstruction::Dup),

            "ILOAD" => {
                let arg = parts.next().ok_or("ILOAD requires numeric argument")?;
                let val: u8 = arg.parse().map_err(|_| "ILOAD argument must be integer")?;
                Ok(IjvmInstruction::ILoad(val))
            }
            _ => Err(format!("unknown opcode: {op}")),
        }
    }
}

impl From<IjvmParseError> for ExecuteError {
    fn from(e: IjvmParseError) -> Self {
        Self::Ijvm(e)
    }
}

impl From<RegisterParseError> for ExecuteError {
    fn from(e: RegisterParseError) -> Self {
        Self::Registers(e)
    }
}

impl std::fmt::Display for IjvmParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::InvalidLine { line, message } => write!(f, "Line {line}: {message}"),
        }
    }
}

impl From<std::io::Error> for ExecuteError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::error::Error for IjvmParseError {}
