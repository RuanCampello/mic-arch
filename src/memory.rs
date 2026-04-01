use std::path::Path;

use crate::microinstruction::MicroInstruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Memory of 16 words in 32-bits indexed by `mar`.
pub struct Memory(pub [u32; 16]);

impl Memory {
    pub fn zero() -> Self {
        Self([0; 16])
    }

    pub fn read(&self, addr: u32) -> u32 {
        self.0[addr as usize]
    }

    pub fn write(&mut self, addr: u32, value: u32) {
        self.0[addr as usize] = value;
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        use std::io::{Error, ErrorKind};

        let contents = std::fs::read_to_string(path.as_ref())?;
        let mut mem = [0u32; 16];
        let mut count = 0;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if count >= 16 {
                return Err(Error::new(ErrorKind::InvalidInput, "too many lines"));
            }

            if line.len() != 32 || !line.chars().all(|c| c == '0' || c == '1') {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "non binary memory segment",
                ));
            }

            mem[count] = u32::from_str_radix(line, 2).unwrap();
            count += 1;
        }

        Ok(Self(mem))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryOperation {
    None = 0b00,
    Read = 0b01,
    Write = 0b10,
    ReadWrite = 0b11,
}

/// Registrador fonte do B-bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BBus {
    Mdr = 0,
    Pc = 1,
    Mbr = 2,
    Mbru = 3,
    Sp = 4,
    Lv = 5,
    Cpp = 6,
    Tos = 7,
    Opc = 8,
    H = 9,
}

impl BBus {
    pub fn name(self) -> &'static str {
        match self {
            BBus::Mdr => "mdr",
            BBus::Pc => "pc",
            BBus::Mbr => "mbr",
            BBus::Mbru => "mbru",
            BBus::Sp => "sp",
            BBus::Lv => "lv",
            BBus::Cpp => "cpp",
            BBus::Tos => "tos",
            BBus::Opc => "opc",
            BBus::H => "h",
        }
    }
}

impl MicroInstruction {
    /// c-bus bit positions (MSB = bit 8, index 0 in the string)
    const C_NAMES: [&'static str; 9] = ["h", "opc", "tos", "cpp", "lv", "sp", "pc", "mdr", "mar"];

    /// nomes dos registradores no c-bus que são escritos neste ciclo.
    pub fn c_bus_targets<'n>(self) -> Vec<&'n str> {
        (0..=8)
            .filter(|&i| (self.c_sel >> (8 - i)) & 1 == 1)
            .map(|i| Self::C_NAMES[i])
            .collect()
    }
}
