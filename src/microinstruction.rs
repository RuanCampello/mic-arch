// src/microinstruction.rs

use crate::alu::Alu;
use crate::memory::{BBus, Memory, MemoryOperation};
use crate::{alu::AluInstruction, register::Registers};
use std::fmt;
use std::str::FromStr;

/// estrutura que representa uma microinstrução decodificada
#[derive(Debug, Clone, Copy)]
pub struct MicroInstruction {
    pub alu: AluInstruction, // comando de 8 bits para a ALU
    pub c_sel: u16,          // máscara de bits de 9 bits para os registradores de destino (C-bus)
    pub memory: MemoryOperation,
    pub b_sel: BBus, // seletor de 4 bits para o registrador de origem (B-bus)
}

#[derive(Debug, Clone, Copy)]
pub struct MicroInstructionLog {
    pub cycle: usize,
    pub instruction: MicroInstruction,
    pub regs_before: Registers,
    pub regs_after: Registers,
    pub memory: Memory,
}

/// Possíveis erros ao tentar converter uma string para MicroInstruction
#[derive(Debug, PartialEq)]
pub enum MicroInstructionParseError {
    InvalidLength(usize),
    NonBinaryChar,
    InvalidMemory(u8),
    InvalidBBus(u8),
}

impl MicroInstruction {
    /// Executa um único ciclo de microinstrução.
    ///
    /// Ordem de operações dentro do ciclo:
    /// 1. MEM=01 → `mdr ← memory[mar]`  (leitura, antes do C-bus)
    /// 2. ALU   → `result = alu(h, b_bus)`
    /// 3. C-bus → escrita dos registradores destino
    /// 4. MEM=10 → `memory[mar_new] ← mdr`  (escrita, usa `mar` já atualizado)
    pub fn execute(&self, regs: &Registers, memory: &Memory) -> (Registers, Memory) {
        let mut new_regs = regs.clone();
        let mut new_mem = memory.clone();

        // 1. leitura de memória (antes do c-bus)
        if self.memory == MemoryOperation::Read {
            new_regs.mdr = memory.read(regs.mar);
        }

        // 2. B-bus
        let b_val: u32 = match self.b_sel {
            BBus::Mdr => regs.mdr,
            BBus::Pc => regs.pc,
            BBus::Mbr => (regs.mbr as i8) as u32, // sign-extend
            BBus::Mbru => regs.mbr as u32,        // zero-extend
            BBus::Sp => regs.sp,
            BBus::Lv => regs.lv,
            BBus::Cpp => regs.cpp,
            BBus::Tos => regs.tos,
            BBus::Opc => regs.opc,
            BBus::H => regs.h,
        };

        // 3. ALU: a = h, b = b_val
        let (_, alu_result) = Alu::execute(regs.h, b_val, self.alu);
        let result = alu_result.s;

        // 4. C-bus writes
        // bits MSB→LSB: h(8) opc(7) tos(6) cpp(5) lv(4) sp(3) pc(2) mdr(1) mar(0)
        if self.c_sel & (1 << 8) != 0 {
            new_regs.h = result;
        }
        if self.c_sel & (1 << 7) != 0 {
            new_regs.opc = result;
        }
        if self.c_sel & (1 << 6) != 0 {
            new_regs.tos = result;
        }
        if self.c_sel & (1 << 5) != 0 {
            new_regs.cpp = result;
        }
        if self.c_sel & (1 << 4) != 0 {
            new_regs.lv = result;
        }
        if self.c_sel & (1 << 3) != 0 {
            new_regs.sp = result;
        }
        if self.c_sel & (1 << 2) != 0 {
            new_regs.pc = result;
        }
        if self.c_sel & (1 << 1) != 0 {
            new_regs.mdr = result;
        }
        if self.c_sel & (1 << 0) != 0 {
            new_regs.mar = result;
        }

        // 5. escrita de memória (usa mar atualizado)
        if self.memory == MemoryOperation::Write {
            new_mem.write(new_regs.mar, new_regs.mdr);
        }

        (new_regs, new_mem)
    }
}

// implementação do display para o Erro (como a mensagem de erro será exibida)
impl fmt::Display for MicroInstructionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(len) => {
                write!(
                    f,
                    "Tamanho inválido: esperado 21 caracteres, mas recebeu {}",
                    len
                )
            }
            Self::NonBinaryChar => {
                write!(
                    f,
                    "A instrução contém caracteres que não são binários (apenas '0' e '1' permitidos)"
                )
            }
            Self::InvalidMemory(mem) => write!(f, "invalid memory value: {mem:#04b}"),
            Self::InvalidBBus(bus) => write!(f, "invalid b_bus value: {bus}"),
        }
    }
}

// implementação básica da trait Error do Rust
impl std::error::Error for MicroInstructionParseError {}

// implementação do "Parser" de String para MicroInstruction
impl FromStr for MicroInstruction {
    type Err = MicroInstructionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1- verifica se a string tem exatamente 21 caracteres
        if s.len() != 23 {
            return Err(MicroInstructionParseError::InvalidLength(s.len()));
        }

        // 2- verifica se a string possui apenas os caracteres '0' e '1'
        if !s.chars().all(|c| c == '0' || c == '1') {
            return Err(MicroInstructionParseError::NonBinaryChar);
        }

        // 3- fatiamento (slicing) e conversão das partes:

        // assume que AluInstruction já possui implementação de FromStr para a string de 8 bits
        // os índices vão de 0 a 7
        let alu = AluInstruction::from_str(&s[0..8])
            .map_err(|_| MicroInstructionParseError::NonBinaryChar)?;

        // pega os índices de 8 a 16 e converte da base 2 para um inteiro u16
        // o unwrap é seguro aqui pois já validamos que só existem '0's e '1's
        let c_sel = u16::from_str_radix(&s[8..17], 2).unwrap();

        let mem_bits = u8::from_str_radix(&s[17..19], 2).unwrap();
        let b_bits = u8::from_str_radix(&s[19..23], 2).unwrap();

        let memory = match mem_bits {
            0b00 => MemoryOperation::None,
            0b01 => MemoryOperation::Read,
            0b10 => MemoryOperation::Write,
            v => return Err(MicroInstructionParseError::InvalidMemory(v)),
        };

        let b_bus = match b_bits {
            0 => BBus::Mdr,
            1 => BBus::Pc,
            2 => BBus::Mbr,
            3 => BBus::Mbru,
            4 => BBus::Sp,
            5 => BBus::Lv,
            6 => BBus::Cpp,
            7 => BBus::Tos,
            8 => BBus::Opc,
            9 => BBus::H,
            v => return Err(MicroInstructionParseError::InvalidBBus(v)),
        };

        Ok(Self {
            alu,
            c_sel,
            b_sel: b_bus,
            memory,
        })
    }
}

// implementação para imprimir a Microinstrução com espaços no formato correto
impl fmt::Display for MicroInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Assume que `self.alu` imprime corretamente seus 8 bits.
        // {:09b} força o c_sel a ser impresso em binário com exatos 9 dígitos.
        // {:04b} força o b_sel a ser impresso em binário com exatos 4 dígitos.
        write!(
            f,
            "{} {:09b} {:02b} {:04b}",
            self.alu, self.c_sel, self.memory as u8, self.b_sel as u8
        )
    }
}

/// retorna uma lista com os nomes dos registradores selecionados pela máscara do Barramento C
pub fn c_bus_names<'n>(c_sel: u16) -> Vec<&'n str> {
    let mut names = Vec::new();

    for bit in 0..=8 {
        // verificamos cada bit fazendo uma operação AND bit-a-bit (Bitwise AND).
        // se o resultado for diferente de 0, significa que o bit está ligado ('1').
        if (c_sel & (1 << bit)) != 0 {
            match bit {
                0 => names.push("MAR"),
                1 => names.push("MDR"),
                2 => names.push("PC"),
                3 => names.push("SP"),
                4 => names.push("LV"),
                5 => names.push("CPP"),
                6 => names.push("TOS"),
                7 => names.push("OPC"),
                8 => names.push("H"),
                _ => {} // ignoramos outros bits
            }
        }
    }

    names
}

/// escreve o resultado `sd` nos registradores apropriados com base na máscara do Barramento C
pub fn c_bus_write(regs: &mut Registers, c_sel: u16, sd: u32) {
    // mesma lógica de validação de bit que a função c_bus_names,
    // mas em vez de retornar strings, salva fisicamente o valor 'sd' no struct Registers.
    for bit in 0..=8 {
        if (c_sel & (1 << bit)) != 0 {
            match bit {
                0 => regs.mar = sd,
                1 => regs.mdr = sd,
                2 => regs.pc = sd,
                3 => regs.sp = sd,
                4 => regs.lv = sd,
                5 => regs.cpp = sd,
                6 => regs.tos = sd,
                7 => regs.opc = sd,
                8 => regs.h = sd,
                _ => {} // ignoramos outros bits
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // importa tudo o que criamos no arquivo
    use std::str::FromStr;

    #[test]
    fn test_c_bus_names_single() {
        // testa se o bit 0 (extrema direita) liga apenas o MAR
        // 0b000000001 é a representação literal binária no Rust
        let names = c_bus_names(0b000000001);
        assert_eq!(names, vec!["MAR"]);
    }

    #[test]
    fn test_c_bus_names_multiple() {
        // testa ligar MAR (bit 0), PC (bit 2) e H (bit 8)
        // 100000101 em binário
        let names = c_bus_names(0b100000101);
        assert_eq!(names, vec!["MAR", "PC", "H"]);
    }

    #[test]
    fn test_parse_invalid_length() {
        // passando uma string com menos de 21 caracteres
        let result = MicroInstruction::from_str("1010");

        assert_eq!(
            result.unwrap_err(),
            MicroInstructionParseError::InvalidLength(4)
        );
    }

    #[test]
    fn test_parse_invalid_chars() {
        // passando 21 caracteres, mas com letras no meio
        let result = MicroInstruction::from_str("00000000A000000000000");

        assert_eq!(
            result.unwrap_err(),
            MicroInstructionParseError::NonBinaryChar
        );
    }
}
