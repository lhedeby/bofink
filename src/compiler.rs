use crate::opcode::OpCode;
use crate::scanner::{Scanner, Token, TokenKind};

struct Compiler {
    chunk: Chunk,
    p: usize,
    locals: Vec<Local>,
    local_count: usize,
    scope_depth: usize,
    tokens: Vec<Token>,
}

pub fn compile2(source: String) -> Chunk {
    let mut compiler = Compiler {
        chunk: Chunk {
            code: vec![],
            line: vec![],
            strings: vec![],
            ints: vec![],
            patch_list: vec![],
        },

        p: 0,
        locals: vec![],
        local_count: 0,
        scope_depth: 0,
        tokens: Scanner::get_tokens(source),
    };

    println!("=== TOKENS ===");
    for i in 0..compiler.tokens.len() {
        println!("{:?}", compiler.tokens[i]);
    }
    compiler.step();
    println!("COMPILING COMPLETED");
    compiler.chunk
}

#[derive(PartialEq, Eq, Debug)]
enum ExpressionKind {
    Bool,
    String,
    Int,
}
impl Compiler {
    fn consume_token(&mut self, kind: TokenKind) {
        if self.tokens[self.p].kind != kind {
            panic!(
                "Expected '{:?}' token but got '{:?}', p: '{}'",
                kind, self.tokens[self.p].kind, self.p
            );
        }
        self.p += 1;
    }

    fn step(&mut self) {
        loop {
            let curr_token = &self.tokens[self.p];
            println!("{:?}", curr_token);
            match curr_token.kind {
                TokenKind::Str => {
                    self.p += 1;
                    let identifier = &self.tokens[self.p].value.to_string();
                    self.consume_token(TokenKind::Identifier);
                    if self.locals.iter().any(|x| &x.name == identifier) {
                        panic!("Cannot redeclare local variable");
                    }
                    self.consume_token(TokenKind::Equal);
                    self.expression();

                    self.locals.push(Local {
                        name: identifier.to_string(),
                        stack_pos: self.local_count,
                        kind: TokenKind::String,
                    });
                    self.local_count += 1;
                    self.consume_token(TokenKind::Semicolon);
                }
                TokenKind::Bool => {
                    self.p += 1;
                    let identifier = &self.tokens[self.p].value.to_string();
                    self.consume_token(TokenKind::Identifier);
                    self.consume_token(TokenKind::Equal);
                    self.expression();
                    println!("Creating local bool");
                    self.locals.push(Local {
                        name: identifier.to_string(),
                        stack_pos: self.local_count,
                        kind: TokenKind::Bool,
                    });
                    self.local_count += 1;
                    self.consume_token(TokenKind::Semicolon);
                }
                TokenKind::Int => {
                    self.p += 1;
                    let identifier = &self.tokens[self.p].value.to_string();
                    self.consume_token(TokenKind::Identifier);
                    self.consume_token(TokenKind::Equal);
                    self.expression();
                    println!("Creating local int");
                    self.locals.push(Local {
                        name: identifier.to_string(),
                        stack_pos: self.local_count,
                        kind: TokenKind::Int,
                    });
                    self.local_count += 1;
                    self.consume_token(TokenKind::Semicolon);
                }
                TokenKind::RightBrace => {
                    self.p += 1;
                    return;
                }

                TokenKind::Eof => return,
                _ => {
                    self.statement();
                }
            }
        }
    }
    fn statement(&mut self) {
        let curr_token = &self.tokens[self.p];
        match curr_token.kind {
            TokenKind::For => {}
            TokenKind::If => {
                self.p += 1;
                self.expression();
                self.consume_token(TokenKind::LeftBrace);
                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk.emit_placeholder(0);
                self.chunk.emit_code(OpCode::JumpIfFalse as u8, 0);
                self.step();
                self.chunk.replace_placeholder();
            }
            TokenKind::Print => {
                let print_token_line = curr_token.line;
                self.p += 1;
                self.expression();
                self.chunk.emit_code(OpCode::Print as u8, print_token_line);
                self.consume_token(TokenKind::Semicolon);
            }
            TokenKind::Return => {}
            TokenKind::While => {}
            // dont know if I should allow arbitrary blocks
            //TokenKind::LeftBrace => {}
            _ => {
                self.expression();
            }
        }
    }
    fn expression(&mut self) {
        let mut previous: Option<ExpressionKind> = None;
        let mut current: Option<ExpressionKind> = None;
        let mut operator: Option<Operator> = None;
        loop {
            let curr_token = &self.tokens[self.p];
            match curr_token.kind {
                TokenKind::True => {
                    self.chunk.emit_code(OpCode::True as u8, curr_token.line);
                    previous = current;
                    current = Some(ExpressionKind::Bool);
                }
                TokenKind::False => {
                    self.chunk.emit_code(OpCode::False as u8, curr_token.line);
                    previous = current;
                    current = Some(ExpressionKind::Bool);
                }
                TokenKind::Number => {
                    self.chunk.emit_number(curr_token);
                    previous = current;
                    current = Some(ExpressionKind::Int);
                }
                TokenKind::String => {
                    self.chunk.emit_string(curr_token);
                    previous = current;
                    current = Some(ExpressionKind::String);
                }
                TokenKind::Identifier => {
                    self.chunk
                        .emit_code(OpCode::GetLocal as u8, curr_token.line);
                    for local in &self.locals {
                        if local.name == curr_token.value {
                            println!("STACK POS: {}", local.stack_pos);
                            self.chunk.emit_code(local.stack_pos as u8, curr_token.line);
                            previous = current;
                            current = match local.kind {
                                TokenKind::String => Some(ExpressionKind::String),
                                TokenKind::Int => Some(ExpressionKind::Int),
                                TokenKind::Bool => Some(ExpressionKind::Bool),
                                _ => panic!("Not a valid local kind (TODO: Bool)"),
                            };
                            break;
                        }
                    }
                }
                TokenKind::Plus => {
                    self.p += 1;
                    operator = Some(Operator::Add);
                    continue;
                }
                TokenKind::Minus => {
                    self.p += 1;
                    operator = Some(Operator::Subtract);
                    continue;
                }
                TokenKind::Star => {
                    self.p += 1;
                    operator = Some(Operator::Multiply);
                    continue;
                }
                TokenKind::Slash => {
                    self.p += 1;
                    operator = Some(Operator::Divide);
                    continue;
                }
                TokenKind::Nil => {
                    todo!("not implemented")
                }
                TokenKind::Semicolon => break,
                TokenKind::LeftBrace => break,
                TokenKind::RightBrace => {
                    self.p += 1;
                    break;
                }
                _ => panic!(
                    "Unexpected token '{:?}' at line {}",
                    curr_token.kind, curr_token.line
                ),
            }

            // adding operators after
            match operator {
                Some(Operator::Add) => match (&previous, &current) {
                    (Some(ExpressionKind::String), Some(ExpressionKind::String)) => {
                        self.chunk
                            .emit_code(OpCode::StringStringConcat as u8, curr_token.line);
                        current = Some(ExpressionKind::String);
                    }
                    (Some(ExpressionKind::String), Some(ExpressionKind::Int)) => {
                        self.chunk
                            .emit_code(OpCode::StringIntConcat as u8, curr_token.line);
                        current = Some(ExpressionKind::String);
                    }
                    (Some(ExpressionKind::Int), Some(ExpressionKind::String)) => {
                        self.chunk
                            .emit_code(OpCode::IntStringConcat as u8, curr_token.line);
                        current = Some(ExpressionKind::String);
                    }
                    (Some(ExpressionKind::String), Some(ExpressionKind::Bool)) => {
                        self.chunk
                            .emit_code(OpCode::StringBoolConcat as u8, curr_token.line);
                        current = Some(ExpressionKind::String);
                    }
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => {
                        self.chunk.emit_code(OpCode::Add as u8, curr_token.line)
                    }
                    (Some(ExpressionKind::Bool), Some(ExpressionKind::String)) => {
                        self.chunk
                            .emit_code(OpCode::BoolStringConcat as u8, curr_token.line);
                        current = Some(ExpressionKind::String);
                    }
                    _ => panic!(
                        "'{:?}' and '{:?}' not valid for add operator.",
                        &previous, &current
                    ),
                },
                Some(Operator::Subtract) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => self
                        .chunk
                        .emit_code(OpCode::Subtract as u8, curr_token.line),

                    _ => panic!(
                        "'{:?}' and '{:?}' not valid for minus operator.",
                        previous, current
                    ),
                },
                Some(Operator::Multiply) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => self
                        .chunk
                        .emit_code(OpCode::Multiply as u8, curr_token.line),

                    _ => panic!(
                        "'{:?}' and '{:?}' not valid for multiply operator.",
                        previous, current
                    ),
                },
                Some(Operator::Divide) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => {
                        self.chunk.emit_code(OpCode::Divide as u8, curr_token.line)
                    }

                    _ => panic!(
                        "'{:?}' and '{:?}' not valid for divide operator.",
                        previous, current
                    ),
                },
                None => {}
            }
            operator = None;
            self.p += 1;
        }
    }
}

struct Local {
    // name: Token,
    // depth: usize,
    kind: TokenKind,
    name: String,
    stack_pos: usize,
}

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub line: Vec<usize>,
    pub strings: Vec<String>,
    pub ints: Vec<i64>,
    pub patch_list: Vec<usize>,
}

enum Operator {
    Add,
    Subtract,
    Divide,
    Multiply,
}

impl Chunk {
    fn emit_placeholder(&mut self, line: usize) {
        self.patch_list.push(self.code.len());
        self.code.push(0);
        self.line.push(line);
        println!("CREATE PLACEHOLDER");
        println!("{:?}", self.patch_list);
        println!("{:?}", self.code);
    }

    fn replace_placeholder(&mut self) {
        if let Some(p) = self.patch_list.pop() {
            let jump_len = self.code.len() - p - 2;
            println!("{:?}", self.code);
            println!("PATCHING JUMP jump len: {}", jump_len);
            self.code[p] = jump_len as u8;
            println!("from: {}, to: {}", p, self.code.len());
        } else {
            panic!("Patch list is empty");
        }
    }

    fn emit_code(&mut self, b: u8, line: usize) {
        self.code.push(b);
        self.line.push(line);
    }
    fn emit_number(&mut self, token: &Token) {
        let int: i64 = token.value.parse().unwrap();
        self.ints.push(int);
        self.emit_code(OpCode::Int as u8, token.line);
        self.emit_code(self.ints.len() as u8 - 1, token.line);
    }
    fn emit_string(&mut self, token: &Token) {
        self.strings.push(token.value.to_string());
        self.emit_code(OpCode::String as u8, token.line);
        self.emit_code(self.strings.len() as u8 - 1, token.line);
    }
}
