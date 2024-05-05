#[repr(u8)]
#[derive(Debug)]
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

