#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    _Constant,
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
    BoolStringConcat,
    StringBoolConcat,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    SetJump,
    JumpBack,
    JumpForward,
    CompareString,
    CompareBool,
    CompareInt,
    CompareStringNot,
    CompareBoolNot,
    CompareIntNot,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    FunctionCall,
    PopStack,
    SetOffset,
    PopOffset,
    Modulo,
    And,
    Or,
    ReturnValue,
    Not,
}

