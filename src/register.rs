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
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> Result<Registers, RegisterParseError> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        let mut registers = Registers::default();

        for (idx, line) in contents.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let (name, value) = line
                .split_once('=')
                .map(|(n, v)| (n.trim(), v.trim()))
                .ok_or(RegisterParseError::InvalidLine { line: idx + 1 })?;

            let parse_u32 = |s: &str| {
                u32::from_str_radix(s, 2)
                    .map_err(|_| RegisterParseError::InvalidLine { line: idx + 1 })
            };

            match name {
                "mar" => registers.mar = parse_u32(value)?,
                "mdr" => registers.mdr = parse_u32(value)?,
                "pc" => registers.pc = parse_u32(value)?,
                "sp" => registers.sp = parse_u32(value)?,
                "lv" => registers.lv = parse_u32(value)?,
                "cpp" => registers.cpp = parse_u32(value)?,
                "tos" => registers.tos = parse_u32(value)?,
                "opc" => registers.opc = parse_u32(value)?,
                "h" => registers.h = parse_u32(value)?,
                "mbr" => {
                    registers.mbr = u8::from_str_radix(value, 2)
                        .map_err(|_| RegisterParseError::InvalidLine { line: idx + 1 })?
                }
                _ => return Err(RegisterParseError::InvalidLine { line: idx + 1 }),
            }
        }

        Ok(registers)
    }

    #[inline(always)]
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

    #[inline(always)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_map_correctly() {
        let registers = Registers {
            mar: 10,
            mdr: 20,
            pc: 30,
            mbr: 0,
            sp: 40,
            lv: 50,
            cpp: 60,
            tos: 70,
            opc: 80,
            h: 90,
        };

        assert_eq!(registers.b_bus_decode(0), 20);
        assert_eq!(registers.b_bus_decode(1), 30);
        assert_eq!(registers.b_bus_decode(4), 40);
        assert_eq!(registers.b_bus_decode(5), 50);
        assert_eq!(registers.b_bus_decode(6), 60);
        assert_eq!(registers.b_bus_decode(7), 70);
        assert_eq!(registers.b_bus_decode(8), 80);
    }

    #[test]
    fn load_registers_any_order() {
        use std::env::temp_dir;
        use std::fs::{remove_file, write};

        let content = r#"
            h = 00000000000000000000000000000001
            mbr = 00000001
            mar = 00000000000000000000000000000100
            mdr = 00000000000000000000000000000000
            pc = 00000000000000000000000000000000
            sp = 00000000000000000000000000000100
            lv = 00000000000000000000000000000000
            cpp = 00000000000000000000000000000000
            tos = 00000000000000000000000000000000
            opc = 00000000000000000000000000000000
        "#;

        let path = temp_dir().join(format!("mic-arch-regs-ord-{}.txt", std::process::id()));
        write(&path, content).unwrap();
        let regs = Registers::load(&path).unwrap();
        let _ = remove_file(&path);

        assert_eq!(regs.h, 1);
        assert_eq!(regs.mar, 4);
        assert_eq!(regs.sp, 4);
    }
}
