#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    Constant,
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Print,
    String,
    Int,
    StringStringConcat,
    StringIntConcat,
    IntStringConcat,
    GetLocal,
    SetLocal,
}

