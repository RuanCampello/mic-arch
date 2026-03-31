// src/microinstruction.rs

use crate::{alu::AluInstruction, register::Registers};
use std::fmt;
use std::str::FromStr;

/// estrutura que representa uma microinstrução decodificada
#[derive(Debug, Clone, Copy)]
pub struct MicroInstruction {
    pub alu: AluInstruction, // comando de 8 bits para a ALU
    pub c_sel: u16,          // máscara de bits de 9 bits para os registradores de destino (C-bus)
    pub b_sel: u8,           // seletor de 4 bits para o registrador de origem (B-bus)
}

/// Possíveis erros ao tentar converter uma string para MicroInstruction
#[derive(Debug, PartialEq)]
pub enum MicroInstructionParseError {
    InvalidLength(usize),
    NonBinaryChar,
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
        if s.len() != 21 {
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

        // pega os índices de 17 a 20 e converte da base 2 para um inteiro u8
        let b_sel = u8::from_str_radix(&s[17..21], 2).unwrap();

        Ok(Self { alu, c_sel, b_sel })
    }
}

// implementação para imprimir a Microinstrução com espaços no formato correto
impl fmt::Display for MicroInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Assume que `self.alu` imprime corretamente seus 8 bits.
        // {:09b} força o c_sel a ser impresso em binário com exatos 9 dígitos.
        // {:04b} força o b_sel a ser impresso em binário com exatos 4 dígitos.
        write!(f, "{} {:09b} {:04b}", self.alu, self.c_sel, self.b_sel)
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

