use crate::alu::{AluInstruction, AluParseError};
use std::{path::Path, str::FromStr};

#[derive(Debug)]
pub enum LoaderError {
    Io(std::io::Error),
    Line { line: usize, message: AluParseError },
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::Io(e)
    }
}

impl std::error::Error for LoaderError {}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::Line { line, message } => write!(
                f,
                "Failed to load program due to Alu parsing error on {line}: {message}"
            ),
            LoaderError::Io(io) => writeln!(f, "Failed to load program due to IO error: {io}"),
        }
    }
}

/// Reads a text file of one control word per line: **6** or **8** binary digits per line
/// (see [`AluInstruction::from_str`](crate::alu::AluInstruction)).
/// Blank lines (including a trailing empty line after the last instruction) are skipped.
pub fn load_program(path: impl AsRef<Path>) -> Result<Vec<AluInstruction>, LoaderError> {
    let contents = std::fs::read_to_string(path.as_ref())?;
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
