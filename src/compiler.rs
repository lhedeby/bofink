use crate::opcode::OpCode;
use crate::scanner::{Scanner, Token, TokenKind};

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
                let identifier = &tokens[p].value;
                p += 1;
                match tokens[p].kind {
                    TokenKind::Equal => {}
                    rest => panic!("Expected 'Equals' token, got '{:?}'", rest),
                }
                if locals.iter().any(|x| &x.name == identifier) {
                    panic!("Cannot redeclare local variable");
                }
                int_expression(&mut p, &mut chunk, &tokens, &locals);
                locals.push(Local {
                    name: identifier.to_string(),
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
    let mut add_add_instruction = false;
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
                    panic!("+ + is not a valid operation");
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
}
