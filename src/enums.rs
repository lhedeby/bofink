use std::fmt;

use crate::scanner::Token;

#[derive(Debug)]
pub enum CompilerError {
    CantMut {
        token: Token,
    },
    Type {
        actual: ExpressionKind,
        expected: ExpressionKind,
        line: usize,
    },
    NotAType {
        kind: TokenKind,
        line: usize
    },
    InvalidToken {
        actual: TokenKind,
        line: usize,
    },
    UnexpectedToken {
        expected: TokenKind,
        actual: TokenKind,
        line: usize,
    },
    Redeclaration(usize),
    DelcarationType {
        expected: ExpressionKind,
        actual: ExpressionKind,
        line: usize,
    },
    MaxFunctions,
    UnknownParamType(usize),
    MissingLocal {
        name: String,
        line: usize,
    },
    ReassignmentType {
        expected: ExpressionKind,
        actual: ExpressionKind,
        line: usize,
    },
    ParamType {
        expected: ExpressionKind,
        actual: ExpressionKind,
        line: usize,
    },
    _NumberOperation {
        operator: TokenKind,
    },
    NoneValue {
        line: usize
    },
    ComparisonType {
        first: ExpressionKind,
        second: ExpressionKind,
        line: usize,
    },
    _InvalidOperatorTypes {
        first: ExpressionKind,
        second: ExpressionKind,
        line: usize,
    },
    ReturnValueFromVoid {
        kind: ExpressionKind,
        line: usize,
    },
    _BooleanExpression(usize),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::CantMut { token } => write!(f, "Cant mutate var | at line {}", token.line),
            CompilerError::NotAType { kind: token, line } => write!(f, "Exptected type annotation but got '{:?}' | at line {}", token, line),
            CompilerError::ReturnValueFromVoid { kind, line } => write!(f, "Cannot return a '{:?}' value from a function that has no return type | at line {}", kind, line),
            CompilerError::NoneValue { line } => write!(f, "Trying to use use 'None' value in expression | at line {}", line),
            CompilerError::Type { actual, expected, line } => write!(f, "Expected type '{:?}' but got '{:?}' | at line {}", expected, actual, line),
            CompilerError::_NumberOperation { operator } => write!(f, "Operator '{:?}' expects 2 numbers", operator),
            CompilerError::InvalidToken { actual, line } => write!(f, "Unexpected token '{:?}' | at line {}", actual, line),
            CompilerError::UnexpectedToken {
                expected,
                actual,
                line,
            } => write!(
                f,
                "Unexpected token | Expected '{:?}' but got '{:?}' | at line {}",
                expected, actual, line
            ),
            CompilerError::Redeclaration(line) => write!(f, "Cannot redeclare variables | at line {}", line),
            CompilerError::DelcarationType {
                expected,
                actual,
                line,
            } => write!(f, "Expression does not match declaration type | Expected '{:?}' but got '{:?}' | at line {}", expected, actual, line),
            CompilerError::MaxFunctions => write!(f, "Too many functions | At the moment bofink only supports {} functions in any program", u8::MAX),
            CompilerError::UnknownParamType(line) => write!(f, "Unexpected paramater type | at line {}", line),
            CompilerError::MissingLocal { name, line } => write!(f, "Could not find local with name '{}' | at line {}", name, line),
            CompilerError::ReassignmentType {
                expected,
                actual,
                line,
            } => write!(f, "Trying to reassign wrong type to local | Expected '{:?}' but got '{:?}' | at line {}", expected, actual, line),
            CompilerError::ParamType {
                expected,
                actual,
                line,
            } => write!(f, "Unexpected type for parameter | Expected '{:?}' but got '{:?}' | at line {}", expected, actual, line),
            CompilerError::ComparisonType {
                first,
                second,
                line,
            } => write!(f, "Invalid comparison types | Got '{:?}' and '{:?}' | at line '{}'",first, second, line),
            CompilerError::_InvalidOperatorTypes {
                first,
                second,
                line,
            } => write!(f, "Invalid types for operator | Got '{:?}' and '{:?} | at line {}'", first, second, line),
            CompilerError::_BooleanExpression(line) => write!(f, "Expected boolean expressions | at line {}", line),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExpressionKind {
    Bool,
    String,
    Int,
    Class(u8),
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Colon,
    Percent,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    New,
    Let,
    Mut,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    True,
    In,
    Int,
    Str,
    Bool,
    While,
    Error,
    Eof,
}
