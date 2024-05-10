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
            match curr_token.kind {
                TokenKind::Str => {
                    self.p += 1;
                    let identifier = &self.tokens[self.p].value.to_string();
                    self.consume_token(TokenKind::Identifier);
                    // if self.locals.iter().any(|x| &x.name == identifier) {
                    //     panic!("Cannot redeclare local variable");
                    // }
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
            TokenKind::If => {}
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
        loop {
            let curr_token = &self.tokens[self.p];
            match curr_token.kind {
                TokenKind::True => {
                    self.chunk.emit_code(OpCode::True as u8, curr_token.line);
                    previous = Some(ExpressionKind::Bool);
                }
                TokenKind::False => {
                    self.chunk.emit_code(OpCode::False as u8, curr_token.line);
                    previous = Some(ExpressionKind::Bool);
                }
                TokenKind::Number => {
                    self.chunk.emit_number(curr_token);
                    previous = Some(ExpressionKind::Int);
                }
                TokenKind::Plus => {
                    self.p += 1;
                    let next_token = &self.tokens[self.p];
                    match next_token.kind {
                        TokenKind::Number => {
                            self.chunk.emit_number(next_token);
                            match previous {
                                Some(ExpressionKind::Int) => {
                                    self.chunk.emit_code(OpCode::Add as u8, next_token.line)
                                }
                                Some(ExpressionKind::String) => self
                                    .chunk
                                    .emit_code(OpCode::StringIntConcat as u8, next_token.line),
                                Some(ExpressionKind::Bool) => {
                                    panic!("Invalid type concatenation 'Bool' + 'Number'")
                                }
                                None => todo!(),
                            }
                        }
                        TokenKind::String => {
                            self.chunk.emit_string(next_token);
                            match previous {
                                Some(ExpressionKind::Int) => self
                                    .chunk
                                    .emit_code(OpCode::IntStringConcat as u8, next_token.line),
                                Some(ExpressionKind::String) => self
                                    .chunk
                                    .emit_code(OpCode::StringStringConcat as u8, next_token.line),
                                Some(ExpressionKind::Bool) => self
                                    .chunk
                                    .emit_code(OpCode::BoolStringConcat as u8, next_token.line),
                                None => panic!("previous is 'None'"),
                            }
                            previous = Some(ExpressionKind::String);
                        }
                        TokenKind::Identifier => {
                            self.chunk
                                .emit_code(OpCode::GetLocal as u8, next_token.line);
                            for local in &self.locals {
                                if local.name == next_token.value {
                                    self.chunk.emit_code(local.stack_pos as u8, next_token.line);
                                    match (&previous, local.kind) {
                                        (Some(ExpressionKind::String), TokenKind::String) => self.chunk.emit_code(OpCode::StringStringConcat as u8, next_token.line),
                                        (Some(ExpressionKind::Int), TokenKind::String) => self.chunk.emit_code(OpCode::IntStringConcat as u8, next_token.line),
                                        (Some(ExpressionKind::String), TokenKind::Int) => self.chunk.emit_code(OpCode::StringIntConcat as u8, next_token.line),
                                    _ => panic!("Not yet implemented or maybe it just shouldnt be possible. Who knows? previous = '{:?}', local = '{:?}'", &previous, local.kind)

                                    }
                                    break;
                                }
                            }
                        }
                        _ => panic!("Unexpected token '{:?}',", &self.tokens[self.p].kind),
                    }
                }
                TokenKind::Minus => match previous {
                    Some(ExpressionKind::Int) => {
                        self.p += 1;
                        let next_token = &self.tokens[self.p];
                        match next_token.kind {
                            TokenKind::Number => self.chunk.emit_number(next_token),
                            TokenKind::Identifier => {
                                self.chunk
                                    .emit_code(OpCode::GetLocal as u8, next_token.line);
                                for local in &self.locals {
                                    if local.name == next_token.value {
                                        self.chunk
                                            .emit_code(local.stack_pos as u8, next_token.line);
                                        if local.kind != TokenKind::Int {
                                            panic!("Not a valid local type for subtraction")
                                        }
                                        break;
                                    }
                                }
                            }
                            _ => panic!("Unexpected token '{:?}' for subtraction.", next_token),
                        }
                        self.chunk
                            .emit_code(OpCode::Subtract as u8, next_token.line)
                    }
                    _ => panic!(
                        "Unexpected kind '{:?}'. Not possible to subtract from.",
                        previous
                    ),
                },
                TokenKind::Star => match previous {
                    Some(ExpressionKind::Int) => {
                        self.p += 1;
                        let next_token = &self.tokens[self.p];
                        match next_token.kind {
                            TokenKind::Number => self.chunk.emit_number(next_token),
                            TokenKind::Identifier => {
                                self.chunk
                                    .emit_code(OpCode::GetLocal as u8, next_token.line);
                                for local in &self.locals {
                                    if local.name == next_token.value {
                                        self.chunk
                                            .emit_code(local.stack_pos as u8, next_token.line);
                                        if local.kind != TokenKind::Int {
                                            panic!("Not a valid local type for subtraction")
                                        }
                                        break;
                                    }
                                }
                            }
                            _ => panic!("Unexpected token '{:?}' for subtraction.", next_token),
                        }
                        self.chunk
                            .emit_code(OpCode::Multiply as u8, next_token.line)
                    }
                    _ => panic!(
                        "Unexpected kind '{:?}'. Not possible to subtract from.",
                        previous
                    ),
                },
                TokenKind::Slash => match previous {
                    Some(ExpressionKind::Int) => {
                        self.p += 1;
                        let next_token = &self.tokens[self.p];
                        match next_token.kind {
                            TokenKind::Number => self.chunk.emit_number(next_token),
                            TokenKind::Identifier => {
                                self.chunk
                                    .emit_code(OpCode::GetLocal as u8, next_token.line);
                                for local in &self.locals {
                                    if local.name == next_token.value {
                                        self.chunk
                                            .emit_code(local.stack_pos as u8, next_token.line);
                                        if local.kind != TokenKind::Int {
                                            panic!("Not a valid local type for subtraction")
                                        }
                                        break;
                                    }
                                }
                            }
                            _ => panic!("Unexpected token '{:?}' for subtraction.", next_token),
                        }
                        self.chunk.emit_code(OpCode::Divide as u8, next_token.line)
                    }
                    _ => panic!(
                        "Unexpected kind '{:?}'. Not possible to subtract from.",
                        previous
                    ),
                },
                TokenKind::String => {
                    self.chunk.emit_string(curr_token);
                    previous = Some(ExpressionKind::String);
                }
                TokenKind::Identifier => {
                    self.chunk
                        .emit_code(OpCode::GetLocal as u8, curr_token.line);
                    for local in &self.locals {
                        if local.name == curr_token.value {
                            println!("STACK POS: {}", local.stack_pos);
                            self.chunk.emit_code(local.stack_pos as u8, curr_token.line);
                            previous = match local.kind {
                                TokenKind::String => Some(ExpressionKind::String),
                                TokenKind::Int => Some(ExpressionKind::Int),
                                _ => panic!("Not a valid local kind (TODO: Bool)"),
                            };
                            break;
                        }
                    }
                }
                TokenKind::Nil => {
                    todo!("not implemented")
                }
                TokenKind::Semicolon => break,
                _ => panic!(
                    "Unexpected token '{:?}' at line {}",
                    curr_token.kind, self.chunk.line[self.p]
                ),
            }
            self.p += 1;
        }
    }
}

// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER
// OLD COMPILER

pub fn compile(source: String) -> Chunk {
    let tokens = Scanner::get_tokens(source);
    println!("=== TOKENS ===");
    for i in 0..tokens.len() {
        println!("{:?}", tokens[i]);
    }
    let mut p = 0;
    let mut chunk = Chunk {
        code: vec![],
        line: vec![],
        strings: vec![],
        ints: vec![],
    };
    let mut locals: Vec<Local> = vec![];
    let mut local_count = 0;
    let mut scope_depth = 0;

    while p < tokens.len() {
        let curr_token = &tokens[p];
        match curr_token.kind {
            TokenKind::Print => {
                let print_token_line = curr_token.line;
                str_expression(&mut p, &mut chunk, &tokens, &locals);
                chunk.emit_code(OpCode::Print as u8, print_token_line)
            }
            TokenKind::LeftBrace => {
                scope_depth += 1;
                scope_depth -= 1;
            }
            TokenKind::Int => {
                p += 1;
                let identifier = &tokens[p];
                if !matches!(identifier.kind, TokenKind::Identifier) {
                    panic!(
                        "Error parsing at line {}. Expected 'Identifier' token.",
                        curr_token.line
                    );
                }
                p += 1;
                if !matches!(tokens[p].kind, TokenKind::Equal) {
                    panic!(
                        "Error parsing at line {}. Expected 'Equals' token.",
                        curr_token.line
                    );
                }
                if locals.iter().any(|x| &x.name == &identifier.value) {
                    panic!("Cannot redeclare local variable");
                }
                int_expression(&mut p, &mut chunk, &tokens, &locals);
                locals.push(Local {
                    name: identifier.value.to_string(),
                    stack_pos: local_count,
                    kind: TokenKind::Int,
                });
                local_count += 1;
            }
            TokenKind::Str => {
                p += 1;
                let identifier = &tokens[p].value;
                p += 1;
                match tokens[p].kind {
                    TokenKind::Equal => {}
                    rest => panic!("Expected 'Equals' token, got '{:?}'", rest),
                }
                if locals.iter().any(|x| &x.name == identifier) {
                    panic!("Cannot redeclare local variable");
                }
                str_expression(&mut p, &mut chunk, &tokens, &locals);
                locals.push(Local {
                    name: identifier.to_string(),
                    stack_pos: local_count,
                    kind: TokenKind::String,
                });
                local_count += 1;
            }
            TokenKind::Eof => {
                return chunk;
            }
            _ => todo!("Token '{:?}' not implementet", curr_token.kind),
        }
        p += 1;
    }
    panic!("Expected eof token at end of file");
}

fn int_expression(p: &mut usize, chunk: &mut Chunk, tokens: &Vec<Token>, locals: &Vec<Local>) {
    let mut instruction: Option<OpCode> = None;
    loop {
        *p += 1;
        let next_token = &tokens[*p];
        match next_token.kind {
            TokenKind::Semicolon => break,
            TokenKind::Number => {
                let int: i64 = next_token.value.parse().unwrap();
                chunk.ints.push(int);
                chunk.emit_code(OpCode::Int as u8, next_token.line);
                chunk.emit_code(chunk.ints.len() as u8 - 1, next_token.line);
            }
            TokenKind::Plus => {
                if instruction.is_some() {
                    panic!("invalid syntax at line {}", next_token.line);
                }
                instruction = Some(OpCode::Add);
                continue;
                // if add_add_instruction {
                //     panic!("+ + is not a valid operation");
                // }
                // add_add_instruction = true;
                // continue;
            }
            TokenKind::Minus => {
                if instruction.is_some() {
                    panic!("invalid syntax at line {}", next_token.line);
                }
                instruction = Some(OpCode::Subtract);
                continue;
            }
            TokenKind::Star => {
                if instruction.is_some() {
                    panic!("invalid syntax at line {}", next_token.line);
                }
                instruction = Some(OpCode::Multiply);
                continue;
            }
            TokenKind::Slash => {
                if instruction.is_some() {
                    panic!("invalid syntax at line {}", next_token.line);
                }
                instruction = Some(OpCode::Divide);
                continue;
            }
            TokenKind::Identifier => {
                chunk.emit_code(OpCode::GetLocal as u8, next_token.line);
                for local in locals {
                    if local.name == next_token.value {
                        chunk.emit_code(local.stack_pos as u8, next_token.line);
                        break;
                    }
                }
            }
            _ => panic!("Error parsing str expression, got '{:?}'", next_token.kind),
        }
        // if add_add_instruction {
        //     add_add_instruction = false;
        //     chunk.emit_code(OpCode::Add as u8, next_token.line);
        // }
        if let Some(ins) = instruction {
            chunk.emit_code(ins as u8, next_token.line);
            instruction = None;
        }
    }
}
fn str_expression(p: &mut usize, chunk: &mut Chunk, tokens: &Vec<Token>, locals: &Vec<Local>) {
    let mut add_concatenate_instruction = false;
    // let mut concant_instruction: Option<OpCode> = None;
    let mut prev_kind: Option<TokenKind> = None;
    let mut curr_kind: Option<TokenKind> = None;
    loop {
        *p += 1;
        let next_token = &tokens[*p];
        match next_token.kind {
            TokenKind::Semicolon => break,
            TokenKind::String | TokenKind::Number => {
                chunk.strings.push(next_token.value.to_string());
                prev_kind = curr_kind;
                curr_kind = Some(next_token.kind);
                chunk.emit_code(OpCode::String as u8, next_token.line);
                chunk.emit_code(chunk.strings.len() as u8 - 1, next_token.line);
            }
            TokenKind::Plus => {
                if add_concatenate_instruction {
                    panic!("'+' '+' is not a valid operation");
                }
                add_concatenate_instruction = true;
                continue;
            }
            TokenKind::Identifier => {
                chunk.emit_code(OpCode::GetLocal as u8, next_token.line);

                for local in locals {
                    if local.name == next_token.value {
                        prev_kind = curr_kind;
                        curr_kind = Some(local.kind);
                        chunk.emit_code(local.stack_pos as u8, next_token.line);
                        break;
                    }
                }
            }
            _ => panic!("Error parsing str expression, got '{:?}'", next_token.kind),
        }
        if add_concatenate_instruction {
            add_concatenate_instruction = false;
            // println!(
            //     "Adding concat instruction for '{:?}' and '{:?}'.",
            //     prev_kind, curr_kind
            // );
            let kind = match (prev_kind, curr_kind) {
                // (Some(TokenKind::Str), Some(TokenKind::Str)) => OpCode::StringStringConcat,
                // (Some(TokenKind::Int), Some(TokenKind::Str)) => OpCode::IntStringConcat,
                // (Some(TokenKind::Str), Some(TokenKind::Int)) => OpCode::StringIntConcat,
                (Some(TokenKind::String), Some(TokenKind::String)) => OpCode::StringStringConcat,
                (Some(TokenKind::Int), Some(TokenKind::String)) => OpCode::IntStringConcat,
                (Some(TokenKind::String), Some(TokenKind::Int)) => OpCode::StringIntConcat,
                _ => panic!("Unkown concat types '{:?}', '{:?}'", prev_kind, curr_kind),
            };
            chunk.emit_code(kind as u8, next_token.line);
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
}

impl Chunk {
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
