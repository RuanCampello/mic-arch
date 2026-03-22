use crate::alu::AluInstruction;

/// Program Counter: guarda o índice da instrução em execução
pub struct ProgramCounter {
    value: usize,
}

/// Instruction Register: armazena a instrução atual (pode estar vazio)
pub struct InstructionRegister {
    instruction: Option<AluInstruction>,
}

impl ProgramCounter {
    // cria um novo Program Counter iniciando em 0
    pub fn new() -> Self {
        ProgramCounter { value: 0 }
    }

    // retorna o valor atual do contador
    pub const fn get(&self) -> usize {
        self.value
    }

    // incrementa o contador em 1
    pub const fn increment(&mut self) {
        self.value += 1;
    }
}

impl InstructionRegister {
    // cria um registrador vazio (None).
    pub fn new() -> Self {
        InstructionRegister { instruction: None }
    }

    // Retorna uma referência para a instrução armazenada, se houver.
    pub fn get(&self) -> Option<&AluInstruction> {
        self.instruction.as_ref()
    }

    // Carrega uma nova instrução, substituindo a anterior.
    pub const fn load(&mut self, instruction: AluInstruction) {
        self.instruction = Some(instruction);
    }
}
