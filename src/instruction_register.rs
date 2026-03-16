crate::alu::AluControl;

// program Counter: guarda o índice da instrução em execução
pub struct ProgramCounter {
    value: usize,
}

impl ProgramCounter {
    // cria um novo Program Counter iniciando em 0
    pub fn new() -> Self {
        ProgramCounter { value: 0 }
    }

    // retorna o valor atual do contador
    pub fn get(&self) -> usize {
        self.value
    }

    // incrementa o contador em 1
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

// instruction Register: armazena a instrução atual (pode estar vazio)
pub struct InstructionRegister {
    instruction: Option<AluControl>,
}

impl InstructionRegister {
    // cria um registrador vazio (None).
    pub fn new() -> Self {
        InstructionRegister { instruction: None }
    }

    // Retorna uma referência para a instrução armazenada, se houver.
    pub fn get(&self) -> Option<&AluControl> {
        self.instruction.as_ref()
    }

    // Carrega uma nova instrução, substituindo a anterior.
    pub fn load(&mut self, instruction: AluControl) {
        self.instruction = Some(instruction);
    }
}
