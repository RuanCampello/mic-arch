use mic_arch::{cpu::Cpu, loader::load_program, logger::Logger};
use std::{fs::File, io::BufWriter, path::Path};

#[test]
fn test_against_given_output() -> Result<(), Box<dyn std::error::Error>> {
    let b: u32 = 0b00000000000000000000000000000001;
    let a: u32 = 0b11111111111111111111111111111111;

    let path = Path::new("./tests/data/programa_etapa1.txt");
    let output = Path::new("./tests/data/saida_etapa1_output.txt");
    let expected_path = Path::new("./tests/data/saída_etapa1.txt");

    let file = File::create(&output)?;

    let mut cpu = Cpu::new();
    let mut logger = Logger::new(BufWriter::new(file));
    logger.start_program(a, b)?;

    let program = load_program(path)?;
    for (cycle, &instruction) in program.iter().enumerate() {
        let result = cpu.execute_cycle(a, b, instruction);
        logger.log_cycle(cycle + 1, &result)?;
    }

    logger.end_program(program.len() + 1)?;

    let output = std::fs::read_to_string(&output)?;
    let expected_output = std::fs::read_to_string(&expected_path)?;

    assert_eq!(output, expected_output, "both outputs should be equal");
    assert_eq!(output.len(), expected_output.len());

    Ok(())
}
