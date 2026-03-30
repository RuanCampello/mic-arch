use std::path::Path;

/// Drives ALU input B from the 4-bit B-bus selector (lowest 4 bits of a 21-bit instruction).
///
/// | Selector | Source | Treatment          |
/// |----------|--------|--------------------|
/// | 0        | MDR    | direct             |
/// | 1        | PC     | direct             |
/// | 2        | MBR    | zero-extend        |
/// | 3        | MBRU   | sign-extend (bit 7)|
/// | 4        | SP     | direct             |
/// | 5        | LV     | direct             |
/// | 6        | CPP    | direct             |
/// | 7        | TOS    | direct             |
/// | 8        | OPC    | direct             |
///
#[derive(Debug, Default)]
pub struct Registers {
    pub mar: u32,
    pub mdr: u32,
    pub pc: u32,
    pub mbr: u8,
    pub sp: u32,
    pub lv: u32,
    pub cpp: u32,
    pub tos: u32,
    pub opc: u32,
    pub h: u32,
}

#[derive(Debug)]
pub enum RegisterParseError {
    Io(std::io::Error),
    InvalidLine { line: usize },
}

impl Registers {
    pub fn b_bus_decode(&self, selector: u8) -> u32 {
        match selector {
            0 => self.mdr,
            1 => self.pc,
            2 => self.mbr as u32,
            3 => self.mbr as i8 as i32 as u32,
            4 => self.sp,
            5 => self.lv,
            6 => self.cpp,
            7 => self.tos,
            8 => self.opc,
            _ => 0,
        }
    }

    pub fn b_bus_name<'s>(selector: u8) -> &'s str {
        match selector {
            0 => "mdr",
            1 => "pc",
            2 => "mbr",
            3 => "mbru",
            4 => "sp",
            5 => "lv",
            6 => "cpp",
            7 => "tos",
            8 => "opc",
            other => unreachable!("invalid selector for regiters: {other}"),
        }
    }
}

impl std::fmt::Display for RegisterParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error reading registers: {e}"),
            Self::InvalidLine { line } => write!(f, "invalid register line at line {line}"),
        }
    }
}

impl std::error::Error for RegisterParseError {}

impl From<std::io::Error> for RegisterParseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
