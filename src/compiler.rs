use std::collections::HashMap;
use std::fmt;

use crate::opcode::OpCode;
use crate::scanner::{Scanner, Token, TokenKind};

struct Compiler {
    chunk: Chunk,
    p: usize,
    locals: Vec<Vec<Local>>,
    local_count: usize,
    tokens: Vec<Token>,
    functions: HashMap<String, Function>,
    scopes: Vec<usize>,
}

// should implement display
#[derive(Debug)]
// should it be public?
pub enum CompilerError {
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

    NumberOperator {
        // TODO: Implement display instead
        operator: Operator,
        first: ExpressionKind,
        second: ExpressionKind,
    },
    ComparisonType {
        first: ExpressionKind,
        second: ExpressionKind,
        line: usize,
    },
    InvalidOperatorTypes {
        first: ExpressionKind,
        second: ExpressionKind,
        line: usize,
    },
    BooleanExpression(usize),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO: Should Tokenkind impl display?
            CompilerError::UnexpectedToken {
                expected,
                actual,
                line,
            } => write!(
                f,
                "Unexpected token | Expected '{:?}' but got '{:?}' | at line {}.",
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
            CompilerError::NumberOperator {
                operator,
                first,
                second,
            } => write!(f, "{:?} operator only usable with ints | Found '{:?}' and '{:?}'", operator, first, second),
            CompilerError::ComparisonType {
                first,
                second,
                line,
            } => write!(f, "Invalid comparison types | Got '{:?}' and '{:?}' | at line '{}'",first, second, line),
            CompilerError::InvalidOperatorTypes {
                first,
                second,
                line,
            } => write!(f, "Invalid types for operator | Got '{:?}' and '{:?} | at line {}'", first, second, line),
            CompilerError::BooleanExpression(line) => write!(f, "Expected boolean expressions | at line {}", line),
        }
    }
}

type Result<T> = std::result::Result<T, CompilerError>;

pub fn compile(source: String) -> Result<Chunk> {
    let mut compiler = Compiler {
        chunk: Chunk {
            code: vec![],
            line: vec![],
            strings: vec![],
            funcs: vec![],
            ints: vec![],
            patch_list: vec![],
        },

        p: 0,
        locals: vec![vec![]],
        functions: HashMap::new(),
        local_count: 0,
        scopes: vec![],
        tokens: Scanner::get_tokens(source),
    };

    compiler.declaration()?;
    Ok(compiler.chunk)
}

impl Compiler {
    fn consume_token(&mut self, kind: TokenKind) -> Result<Token> {
        let token = &self.tokens[self.p];
        if token.kind != kind {
            return Err(CompilerError::UnexpectedToken {
                expected: kind,
                actual: token.kind,
                line: token.line,
            });
        }
        self.p += 1;
        return Ok(Token {
            kind: token.kind,
            line: token.line,
            value: token.value.to_string(),
        });
    }

    fn consume_if_match(&mut self, kind: TokenKind) -> Option<Token> {
        let token = &self.tokens[self.p];
        if token.kind == kind {
            self.p += 1;
            return Some(Token {
                kind: token.kind,
                line: token.line,
                value: token.value.to_string(),
            });
        }
        None
    }

    fn local_declaration(&mut self, exp_kind: ExpressionKind) -> Result<()> {
        self.p += 1;
        let consumed_token = self.consume_token(TokenKind::Identifier)?;
        let identifier = &consumed_token.value.to_string();
        if self
            .locals
            .last()
            .unwrap()
            .iter()
            .any(|x| &x.name == identifier)
        {
            return Err(CompilerError::Redeclaration(consumed_token.line));
        }
        let consumed_token = self.consume_token(TokenKind::Equal)?;
        let kind = self.expression()?;

        if kind != exp_kind {
            return Err(CompilerError::DelcarationType {
                expected: kind,
                actual: exp_kind,
                line: consumed_token.line,
            });
        }

        self.add_local(identifier, exp_kind);
        self.consume_token(TokenKind::Semicolon)?;
        Ok(())
    }

    fn declaration(&mut self) -> Result<()> {
        loop {
            let curr_token = &self.tokens[self.p];
            match curr_token.kind {
                TokenKind::Str => {
                    self.local_declaration(ExpressionKind::String)?;
                }
                TokenKind::Bool => {
                    self.local_declaration(ExpressionKind::Bool)?;
                }
                TokenKind::Int => {
                    self.local_declaration(ExpressionKind::Int)?;
                }
                // vad ar detta????
                // end scope bara losa allt
                TokenKind::RightBrace => {
                    self.p += 1;
                    return Ok(());
                }
                // Function declaration
                TokenKind::Fun => {
                    self.p += 1;
                    self.locals.push(vec![]);
                    self.local_count = 0;
                    let identifier = &self.tokens[self.p].value.to_string();
                    self.chunk.emit_code(OpCode::SetJump as u8, 0);
                    self.chunk.emit_placeholder(0);
                    self.chunk.emit_code(OpCode::JumpForward as u8, 0);
                    let fun_start = self.chunk.code.len();
                    self.chunk.funcs.push(fun_start);
                    let fun_count = self.functions.len();
                    if fun_count >= u8::MAX as usize {
                        return Err(CompilerError::MaxFunctions);
                    }
                    self.p += 1;
                    self.consume_token(TokenKind::LeftParen)?;
                    let mut function = Function {
                        index: fun_count as u8,
                        params: vec![],
                    };
                    while self.tokens[self.p].kind != TokenKind::RightParen {
                        let consumed_token = self.consume_token(TokenKind::Identifier)?;
                        let param_name = &consumed_token.value.to_string();
                        self.consume_token(TokenKind::Colon)?;
                        let param_kind = match self.tokens[self.p].kind {
                            TokenKind::Int => ExpressionKind::Int,
                            TokenKind::Bool => ExpressionKind::Bool,
                            TokenKind::Str => ExpressionKind::String,
                            _ => {
                                return Err(CompilerError::UnknownParamType(
                                    self.tokens[self.p].line,
                                ))
                            }
                        };
                        self.p += 1;
                        function.params.push(Param { kind: param_kind });
                        self.add_local(param_name, param_kind);
                        self.consume_if_match(TokenKind::Comma);
                    }

                    self.functions.insert(identifier.to_string(), function);
                    self.consume_token(TokenKind::RightParen)?;
                    self.consume_token(TokenKind::LeftBrace)?;
                    self.declaration()?;
                    self.chunk.emit_code(OpCode::Return as u8, 0);
                    self.chunk.replace_placeholder();
                    self.locals.pop();
                    self.local_count = self.locals.last().unwrap().len();
                }

                TokenKind::Eof => return Ok(()),
                _ => {
                    self.statement()?;
                }
            }
        }
    }

    fn start_scope(&mut self) -> Result<()> {
        self.consume_token(TokenKind::LeftBrace)?;
        self.scopes
            .push(self.locals.last().expect("Locals is empty.").len());
        Ok(())
    }

    fn end_scope(&mut self) {
        let end_locals = self.locals.last().expect("Locals is empty.").len();
        let start_locals = self.scopes.pop().expect("No scope exists.");
        for _ in 0..(end_locals - start_locals) {
            self.locals.last_mut().expect("Locals is empty.").pop();
            self.chunk
                .emit_code(OpCode::PopStack as u8, self.tokens[self.p].line);
        }
    }

    fn add_local(&mut self, name: &str, kind: ExpressionKind) {
        self.locals.last_mut().unwrap().push(Local {
            name: name.to_string(),
            stack_pos: self.local_count,
            kind,
        });
        self.local_count += 1;
    }

    fn statement(&mut self) -> Result<()> {
        let curr_token = &self.tokens[self.p];
        match curr_token.kind {
            TokenKind::For => {
                self.p += 1;
                let consumed_token = self.consume_token(TokenKind::Identifier)?;
                let iter_name = &consumed_token.value.to_string();
                self.consume_token(TokenKind::In)?;
                let consumed_token = self.consume_token(TokenKind::Number)?;
                let loop_start = &consumed_token.value.parse::<i64>().unwrap();

                // Emit local
                self.chunk.emit_number(&consumed_token);
                self.add_local(iter_name, ExpressionKind::Int);

                let jump_point = self.chunk.code.len();

                // move on
                self.consume_token(TokenKind::Colon)?;

                // Get the local
                // TODO: Dont need to check. It should always be the last variable
                self.chunk.emit_code(OpCode::GetLocal as u8, 0);
                let mut found = false;
                for local in self.locals.last().unwrap() {
                    if &local.name == iter_name {
                        self.chunk.emit_code(local.stack_pos as u8, 0);
                        found = true;
                        break;
                    }
                }
                if !found {
                    // should also be removed
                    return Err(CompilerError::MissingLocal {
                        name: iter_name.to_string(),
                        line: 0,
                    });
                }

                // push the max to stack
                let consumed_token = self.consume_token(TokenKind::Number)?;

                let loop_end = &consumed_token.value.parse::<i64>().unwrap();
                self.chunk.emit_number(&consumed_token);
                if loop_start <= loop_end {
                    self.chunk
                        .emit_code(OpCode::Less as u8, consumed_token.line);
                } else {
                    self.chunk
                        .emit_code(OpCode::Greater as u8, consumed_token.line);
                }

                // Setup jump
                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk.emit_placeholder(0);
                self.chunk.emit_code(OpCode::JumpIfFalse as u8, 0);

                let mut step = 0;
                let mut negative_increment = false;

                // check for custom increment
                if let Some(_) = self.consume_if_match(TokenKind::Colon) {
                    if let Some(_) = self.consume_if_match(TokenKind::Minus) {
                        negative_increment = true;
                    }
                    let consumed_token = self.consume_token(TokenKind::Number)?;
                    step = consumed_token.value.parse::<i64>().unwrap();
                    if negative_increment {
                        step *= -1;
                    }
                }
                self.start_scope()?;
                self.declaration()?;
                self.end_scope();

                // add increment to local
                // Get the local
                // TODO: loop lookup should also be removed? no need to check cause we know it
                // exists. Maybe lookup should be a seperate function aswell
                self.chunk.emit_code(OpCode::GetLocal as u8, 0);
                let mut found = false;
                for local in self.locals.last().unwrap() {
                    if &local.name == iter_name {
                        self.chunk.emit_code(local.stack_pos as u8, 0);
                        found = true;
                        break;
                    }
                }
                if !found {
                    // this should be removed aswell
                    return Err(CompilerError::MissingLocal {
                        name: iter_name.to_string(),
                        line: 0,
                    });
                }
                // Push the increment to the stack
                if step == 0 {
                    if loop_start <= loop_end {
                        step = 1;
                    } else {
                        step = -1
                    }
                }
                // TODO: Dont use dummy token
                let dummy_token = Token {
                    value: step.to_string(),
                    kind: TokenKind::Number,
                    line: 0,
                };
                self.chunk.emit_number(&dummy_token);
                self.chunk.emit_code(OpCode::Add as u8, 0);
                // set the local
                for local in self.locals.last().unwrap() {
                    if &local.name == iter_name {
                        self.chunk.emit_code(OpCode::SetLocal as u8, 0);
                        self.chunk.emit_code(local.stack_pos as u8, 0);
                        break;
                    }
                }

                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk
                    .emit_code((self.chunk.code.len() - jump_point + 2) as u8, 0);
                self.chunk.emit_code(OpCode::JumpBack as u8, 0);
                self.chunk.replace_placeholder();
            }
            TokenKind::While => {
                let jump_point = self.chunk.code.len();
                self.p += 1;
                self.expression()?;
                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk.emit_placeholder(0);
                self.chunk.emit_code(OpCode::JumpIfFalse as u8, 0);
                self.start_scope()?;
                self.declaration()?;
                self.end_scope();
                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk
                    .emit_code((self.chunk.code.len() - jump_point + 2) as u8, 0);
                self.chunk.emit_code(OpCode::JumpBack as u8, 0);
                self.chunk.replace_placeholder();
            }
            TokenKind::If => {
                self.p += 1;
                self.expression()?;
                self.chunk.emit_code(OpCode::SetJump as u8, 0);
                self.chunk.emit_placeholder(0);
                self.chunk.emit_code(OpCode::JumpIfFalse as u8, 0);
                self.start_scope()?;
                self.declaration()?;
                self.end_scope();
                self.chunk.replace_placeholder();
            }
            TokenKind::Print => {
                let print_token_line = curr_token.line;
                self.p += 1;
                self.expression()?;
                self.chunk.emit_code(OpCode::Print as u8, print_token_line);
                self.consume_token(TokenKind::Semicolon)?;
            }
            TokenKind::Return => {}
            // dont know if I should allow arbitrary blocks
            //TokenKind::LeftBrace => {}
            TokenKind::Identifier => {
                let identifier_name = curr_token.value.to_string();
                let line = curr_token.line;
                self.p += 1;
                match &self.tokens[self.p].kind {
                    // Reassignment
                    TokenKind::Equal => {
                        self.p += 1;
                        let kind = self.expression()?;
                        for local in self.locals.last().unwrap() {
                            if local.name == identifier_name {
                                if local.kind != kind {
                                    return Err(CompilerError::ReassignmentType {
                                        expected: local.kind,
                                        actual: kind,
                                        line: self.tokens[self.p].line,
                                    });
                                }
                                self.chunk.emit_code(OpCode::SetLocal as u8, line);
                                self.chunk.emit_code(local.stack_pos as u8, line);
                                break;
                            }
                        }
                    }
                    // function call
                    TokenKind::LeftParen => {
                        self.consume_token(TokenKind::LeftParen)?;
                        let function = self.functions[&identifier_name].clone();
                        for param in function.params.clone() {
                            let kind = self.expression().unwrap();
                            if kind != param.kind {
                                return Err(CompilerError::ParamType {
                                    expected: param.kind,
                                    actual: kind,
                                    // TODO: self.current_line() to get the current line
                                    line: self.tokens[self.p].line,
                                });
                            }
                            self.local_count += 1;
                            self.consume_if_match(TokenKind::Comma);
                        }
                        self.chunk.emit_code(OpCode::SetOffset as u8, 0);
                        self.chunk.emit_code(function.params.len() as u8, 0);
                        self.chunk.emit_code(OpCode::FunctionCall as u8, line);
                        self.chunk.emit_code(function.index, line);
                        self.consume_token(TokenKind::RightParen)?;
                        for _ in 0..function.params.len() {
                            self.chunk.emit_code(OpCode::PopStack as u8, line);
                        }
                        self.chunk.emit_code(OpCode::PopOffset as u8, 0);
                    }
                    _ => {
                        // Is this even possible? maybe just panic
                        return Err(CompilerError::UnexpectedToken {
                            // TODO: self.get_curr_token()?
                            actual: self.tokens[self.p].kind,
                            expected: TokenKind::Nil,
                            line: self.tokens[self.p].line,
                        });
                    }
                }
                self.consume_token(TokenKind::Semicolon)?;
            }
            _ => {
                self.expression()?;
            }
        }
        Ok(())
    }
    fn expression(&mut self) -> Result<ExpressionKind> {
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
                    let mut found = false;
                    // TODO: Function
                    for local in self.locals.last().unwrap() {
                        if local.name == curr_token.value {
                            self.chunk.emit_code(local.stack_pos as u8, curr_token.line);
                            previous = current;
                            current = Some(local.kind);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Err(CompilerError::MissingLocal {
                            name: curr_token.value.to_string(),
                            line: curr_token.line,
                        });
                    }
                }
                TokenKind::Percent => {
                    self.p += 1;
                    operator = Some(Operator::Modulo);
                    continue;
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
                TokenKind::EqualEqual => {
                    self.p += 1;
                    operator = Some(Operator::EqualEqual);
                    continue;
                }
                TokenKind::BangEqual => {
                    self.p += 1;
                    operator = Some(Operator::BangEqual);
                    continue;
                }
                TokenKind::Less => {
                    self.p += 1;
                    operator = Some(Operator::Less);
                    continue;
                }
                TokenKind::LessEqual => {
                    self.p += 1;
                    operator = Some(Operator::LessEqual);
                    continue;
                }
                TokenKind::Greater => {
                    self.p += 1;
                    operator = Some(Operator::Greater);
                    continue;
                }
                TokenKind::GreaterEqual => {
                    self.p += 1;
                    operator = Some(Operator::GreaterEqual);
                    continue;
                }
                TokenKind::And => {
                    self.p += 1;
                    operator = Some(Operator::And);
                    continue;
                }
                TokenKind::Or => {
                    self.p += 1;
                    operator = Some(Operator::Or);
                    continue;
                }
                TokenKind::Semicolon => break,
                TokenKind::LeftBrace => break,
                TokenKind::Comma => break,
                TokenKind::RightParen => break,
                TokenKind::RightBrace => {
                    self.p += 1;
                    break;
                }
                _ => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: TokenKind::Nil,
                        actual: curr_token.kind,
                        line: curr_token.line,
                    })
                }
            }

            // adding operators after
            match operator {
                Some(Operator::Modulo) => {
                    if &previous != &Some(ExpressionKind::Int)
                        || &current != &Some(ExpressionKind::Int)
                    {
                        // TODO: Is it possible for these to be none? unwrap and check earlier?
                        return Err(CompilerError::NumberOperator {
                            operator: Operator::Modulo,
                            first: previous.unwrap(),
                            second: current.unwrap(),
                        });
                    }
                    self.chunk.emit_code(OpCode::Modulo as u8, curr_token.line);
                    current = Some(ExpressionKind::Int);
                }
                Some(Operator::Greater) => {
                    if &previous != &Some(ExpressionKind::Int)
                        || &current != &Some(ExpressionKind::Int)
                    {
                        return Err(CompilerError::NumberOperator {
                            operator: Operator::Greater,
                            first: previous.unwrap(),
                            second: current.unwrap(),
                        });
                    }
                    self.chunk.emit_code(OpCode::Greater as u8, curr_token.line);
                    current = Some(ExpressionKind::Bool);
                }
                // TODO: Make this 1 big patter (gt | g | modul... etc) and then use
                // operator.get_opcode to get the code to emit?
                Some(Operator::GreaterEqual) => {
                    if &previous != &Some(ExpressionKind::Int)
                        || &current != &Some(ExpressionKind::Int)
                    {
                        return Err(CompilerError::NumberOperator {
                            operator: Operator::GreaterEqual,
                            first: previous.unwrap(),
                            second: current.unwrap(),
                        });
                    }
                    self.chunk
                        .emit_code(OpCode::GreaterEqual as u8, curr_token.line);
                    current = Some(ExpressionKind::Bool);
                }
                Some(Operator::Less) => {
                    if &previous != &Some(ExpressionKind::Int)
                        || &current != &Some(ExpressionKind::Int)
                    {
                        return Err(CompilerError::NumberOperator {
                            operator: Operator::Less,
                            first: previous.unwrap(),
                            second: current.unwrap(),
                        });
                    }
                    self.chunk.emit_code(OpCode::Less as u8, curr_token.line);
                    current = Some(ExpressionKind::Bool);
                }
                Some(Operator::LessEqual) => {
                    if &previous != &Some(ExpressionKind::Int)
                        || &current != &Some(ExpressionKind::Int)
                    {
                        return Err(CompilerError::NumberOperator {
                            operator: Operator::LessEqual,
                            first: previous.unwrap(),
                            second: current.unwrap(),
                        });
                    }
                    self.chunk
                        .emit_code(OpCode::LessEqual as u8, curr_token.line);
                    current = Some(ExpressionKind::Bool);
                }
                Some(Operator::BangEqual) => {
                    if &previous != &current {
                        // TODO: remove unwraps
                        return Err(CompilerError::ComparisonType {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                    match &current {
                        Some(ExpressionKind::String) => {
                            self.chunk
                                .emit_code(OpCode::CompareStringNot as u8, curr_token.line);
                        }
                        Some(ExpressionKind::Bool) => {
                            self.chunk
                                .emit_code(OpCode::CompareBoolNot as u8, curr_token.line);
                        }
                        Some(ExpressionKind::Int) => {
                            self.chunk
                                .emit_code(OpCode::CompareIntNot as u8, curr_token.line);
                        }
                        None => unreachable!("Cant compare none"),
                    }
                    current = Some(ExpressionKind::Bool);
                }
                Some(Operator::EqualEqual) => {
                    if &previous != &current {
                        // TODO: remove unwraps
                        return Err(CompilerError::ComparisonType {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                    match &current {
                        Some(ExpressionKind::String) => {
                            self.chunk
                                .emit_code(OpCode::CompareString as u8, curr_token.line);
                        }
                        Some(ExpressionKind::Bool) => {
                            self.chunk
                                .emit_code(OpCode::CompareBool as u8, curr_token.line);
                        }
                        Some(ExpressionKind::Int) => {
                            self.chunk
                                .emit_code(OpCode::CompareInt as u8, curr_token.line);
                        }
                        None => unreachable!("Cant compare none"),
                    }
                    current = Some(ExpressionKind::Bool);
                }
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
                    _ => {
                        // TODO: remove unwraps and fix line
                        return Err(CompilerError::InvalidOperatorTypes {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                },
                Some(Operator::Subtract) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => self
                        .chunk
                        .emit_code(OpCode::Subtract as u8, curr_token.line),

                    _ => {
                        // TODO: remove unwraps and fix line
                        return Err(CompilerError::InvalidOperatorTypes {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                },
                Some(Operator::Multiply) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => self
                        .chunk
                        .emit_code(OpCode::Multiply as u8, curr_token.line),

                    _ => {
                        // TODO: remove unwraps and fix line
                        return Err(CompilerError::InvalidOperatorTypes {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                },
                Some(Operator::Divide) => match (&previous, &current) {
                    (Some(ExpressionKind::Int), Some(ExpressionKind::Int)) => {
                        self.chunk.emit_code(OpCode::Divide as u8, curr_token.line)
                    }
                    _ => {
                        // TODO: remove unwraps and fix line
                        return Err(CompilerError::InvalidOperatorTypes {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                },
                Some(Operator::And) => match (&previous, &current) {
                    (Some(ExpressionKind::Bool), Some(ExpressionKind::Bool)) => {
                        self.chunk.emit_code(OpCode::And as u8, curr_token.line);
                    }
                    // TODO: Maybe change all these to a generic one? 'ExpressionType' error?
                    _ => return Err(CompilerError::BooleanExpression(curr_token.line)),
                },
                Some(Operator::Or) => match (&previous, &current) {
                    (Some(ExpressionKind::Bool), Some(ExpressionKind::Bool)) => {
                        self.chunk.emit_code(OpCode::Or as u8, curr_token.line);
                    }
                    _ => return Err(CompilerError::BooleanExpression(curr_token.line)),
                },
                None => {}
            }
            operator = None;
            self.p += 1;
        }
        Ok(current.unwrap())
    }
}

#[derive(Debug)]
struct Local {
    kind: ExpressionKind,
    name: String,
    stack_pos: usize,
}

// TODO: derive display instead
#[derive(Debug)]
enum Operator {
    Add,
    Subtract,
    Divide,
    Modulo,
    Multiply,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum ExpressionKind {
    Bool,
    String,
    Int,
}

#[derive(Clone)]
struct Function {
    index: u8,
    params: Vec<Param>,
}

#[derive(Clone)]
struct Param {
    kind: ExpressionKind,
}

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub line: Vec<usize>,
    pub strings: Vec<String>,
    pub ints: Vec<i64>,
    pub patch_list: Vec<usize>,
    pub funcs: Vec<usize>,
}

impl Chunk {
    fn emit_placeholder(&mut self, line: usize) {
        self.patch_list.push(self.code.len());
        self.code.push(0);
        self.line.push(line);
    }

    fn replace_placeholder(&mut self) {
        if let Some(p) = self.patch_list.pop() {
            let jump_len = self.code.len() - p - 2;
            self.code[p] = jump_len as u8;
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
