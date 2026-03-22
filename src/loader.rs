use crate::alu::{AluInstruction, AluParseError};
use std::str::FromStr;

#[derive(Debug)]
pub(crate) enum LoaderError {
    Io(std::io::Error),
    Line { line: usize, message: AluParseError },
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::Io(e)
    }
}

/// Reads a text file of one 6-bit ALU control word per line.
/// Blank lines (including a trailing empty line after the last instruction) are skipped.
pub(crate) fn load_program(path: &str) -> Result<Vec<AluInstruction>, LoaderError> {
    let contents = std::fs::read_to_string(path)?;
    let mut program = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }

        let instr = AluInstruction::from_str(line).map_err(|message| LoaderError::Line {
            line: idx + 1,
            message,
        })?;
        program.push(instr);
    }

    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::{remove_file, write};

    #[test]
    fn load_program_skips_blank_lines() {
        let path = temp_dir().join(format!("mic-arch-loader-{}.txt", std::process::id()));
        write(&path, "111110\n\n110101\n").unwrap();
        let program = load_program(path.to_str().unwrap()).unwrap();

        assert_eq!(program.len(), 2);
        assert_eq!(program[0], AluInstruction::from(0b111110));
        assert_eq!(program[1], AluInstruction::from(0b110101));

        let _ = remove_file(&path);
    }

    #[test]
    fn load_program_reports_bad_line() {
        let path = temp_dir().join(format!("mic-arch-loader-bad-{}.txt", std::process::id()));
        write(&path, "111110\nbad\n").unwrap();

        let err = load_program(path.to_str().unwrap()).unwrap_err();
        match err {
            LoaderError::Line { line, .. } => assert_eq!(line, 2),
            LoaderError::Io(_) => panic!("expected Line error"),
        }

        let _ = remove_file(&path);
    }
}
