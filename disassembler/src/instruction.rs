#[derive(PartialEq, Eq, Debug)]
pub enum InstructionSet {
    HALT,
    SET,
    PUSH,
    POP,
    EQ,
    GT,
    JMP,
    JT,
    JF,
    ADD,
    MULT,
    MOD,
    AND,
    OR,
    NOT,
    RMEM,
    WMEM,
    CALL,
    RET,
    OUT,
    IN,
    NOOP
}

pub enum Type {
    StaticCall,
    DynamicCall,
    Unknown
}

pub struct Instruction {
    pub instr: InstructionSet,
    pub size: u16,
    pub typ: Type,
    pub expr: Option<&'static str>,
}

impl Instruction{
    pub fn new(instr: InstructionSet) -> Instruction {
        let size = match instr {
            InstructionSet::HALT|
            InstructionSet::RET|
            InstructionSet::NOOP => 1,
            InstructionSet::PUSH|
            InstructionSet::POP|
            InstructionSet::JMP|
            InstructionSet::CALL|
            InstructionSet::OUT|
            InstructionSet::IN => 2,
            InstructionSet::SET|
            InstructionSet::JT|
            InstructionSet::JF|
            InstructionSet::NOT|
            InstructionSet::RMEM|
            InstructionSet::WMEM  => 3,
            InstructionSet::EQ|
            InstructionSet::GT|
            InstructionSet::ADD|
            InstructionSet::MULT|
            InstructionSet::MOD|
            InstructionSet::AND|
            InstructionSet::OR => 4,
        };
        Instruction {
            instr: instr,
            size: size,
            typ: Type::Unknown,
            expr: None
        }
    }
}
