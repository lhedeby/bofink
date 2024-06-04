use std::collections::HashMap;

use crate::enums::{CompilerError, ExpressionKind, Operator, TokenKind};

use crate::opcode::OpCode;
use crate::scanner::{Scanner, Token};

struct Compiler {
    chunk: Chunk,
    p: usize,
    locals: Vec<Vec<Local>>,
    local_count: usize,
    tokens: Vec<Token>,
    functions: HashMap<String, Function>,
    scopes: Vec<usize>,
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
                expected: exp_kind,
                actual: kind,
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
                    self.emit_opcode(OpCode::SetJump);
                    // todo self placeholder?
                    self.chunk.emit_placeholder(0);
                    self.emit_opcode(OpCode::JumpForward);
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
                    while self.current_kind() != TokenKind::RightParen {
                        let consumed_token = self.consume_token(TokenKind::Identifier)?;
                        let param_name = &consumed_token.value.to_string();
                        self.consume_token(TokenKind::Colon)?;
                        let param_kind = match self.current_kind() {
                            TokenKind::Int => ExpressionKind::Int,
                            TokenKind::Bool => ExpressionKind::Bool,
                            TokenKind::Str => ExpressionKind::String,
                            _ => return Err(CompilerError::UnknownParamType(self.current_line())),
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
                    self.emit_opcode(OpCode::Return);
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
            self.emit_opcode(OpCode::PopStack);
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
                self.emit_opcode(OpCode::GetLocal);
                let iterator_stack_pos = self
                    .locals
                    .last()
                    .expect("Should always be the last added local-stack")
                    .last()
                    .expect("Should always be the last added local")
                    .stack_pos as u8;
                self.emit_u8(iterator_stack_pos);

                // push the max to stack
                let consumed_token = self.consume_token(TokenKind::Number)?;

                let loop_end = &consumed_token.value.parse::<i64>().unwrap();
                // TODO: emit number self()?
                self.chunk.emit_number(&consumed_token);
                if loop_start <= loop_end {
                    self.emit_opcode(OpCode::Less);
                } else {
                    self.emit_opcode(OpCode::Greater);
                }

                // Setup jump
                self.emit_opcode(OpCode::SetJump);
                self.chunk.emit_placeholder(0);
                self.emit_opcode(OpCode::JumpIfFalse);

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

                self.emit_opcode(OpCode::GetLocal);
                self.emit_u8(iterator_stack_pos);

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
                // todo: self.emit_number?
                self.chunk.emit_number(&dummy_token);
                self.emit_opcode(OpCode::Add);
                self.emit_opcode(OpCode::SetLocal);
                self.emit_u8(iterator_stack_pos);
                self.emit_opcode(OpCode::SetJump);
                self.emit_u8((self.chunk.code.len() - jump_point + 2) as u8);
                self.emit_opcode(OpCode::JumpBack);
                self.chunk.replace_placeholder();
            }
            TokenKind::While => {
                let jump_point = self.chunk.code.len();
                self.p += 1;
                self.expression()?;
                self.emit_opcode(OpCode::SetJump);
                self.chunk.emit_placeholder(0);
                self.emit_opcode(OpCode::JumpIfFalse);
                self.start_scope()?;
                self.declaration()?;
                self.end_scope();
                self.emit_opcode(OpCode::SetJump);
                self.emit_u8((self.chunk.code.len() - jump_point + 2) as u8);
                self.emit_opcode(OpCode::JumpBack);
                self.chunk.replace_placeholder();
            }
            TokenKind::If => {
                self.p += 1;
                self.expression()?;
                self.emit_opcode(OpCode::SetJump);
                self.chunk.emit_placeholder(0);
                self.emit_opcode(OpCode::JumpIfFalse);
                self.start_scope()?;
                self.declaration()?;
                self.end_scope();
                self.chunk.replace_placeholder();
            }
            TokenKind::Print => {
                self.p += 1;
                self.expression()?;
                self.emit_opcode(OpCode::Print);
                self.consume_token(TokenKind::Semicolon)?;
            }
            // TODO I Guess
            TokenKind::Return => {}
            // dont know if I should allow arbitrary blocks
            //TokenKind::LeftBrace => {}
            TokenKind::Identifier => {
                let identifier_name = curr_token.value.to_string();
                self.p += 1;
                match &self.current_kind() {
                    // Reassignment
                    TokenKind::Equal => {
                        self.p += 1;
                        let kind = self.expression()?;

                        self.emit_opcode(OpCode::SetLocal);
                        for local in self.locals.last().unwrap() {
                            if local.name == identifier_name {
                                if local.kind != kind {
                                    return Err(CompilerError::ReassignmentType {
                                        expected: local.kind,
                                        actual: kind,
                                        line: self.tokens[self.p].line,
                                    });
                                }
                                self.emit_u8(local.stack_pos as u8);
                                break;
                            }
                        }
                    }
                    // function call
                    TokenKind::LeftParen => {
                        self.consume_token(TokenKind::LeftParen)?;
                        let function = self.functions[&identifier_name].clone();
                        for param in function.params.clone() {
                            let kind = self.expression()?;
                            if kind != param.kind {
                                return Err(CompilerError::ParamType {
                                    expected: param.kind,
                                    actual: kind,
                                    line: self.current_line(),
                                });
                            }
                            self.local_count += 1;
                            self.consume_if_match(TokenKind::Comma);
                        }

                        self.emit_opcode(OpCode::SetOffset);
                        self.emit_u8(function.params.len() as u8);
                        self.emit_opcode(OpCode::FunctionCall);
                        self.emit_u8(function.index);

                        self.consume_token(TokenKind::RightParen)?;
                        for _ in 0..function.params.len() {
                            self.emit_opcode(OpCode::PopStack);
                        }
                        self.emit_opcode(OpCode::PopOffset);
                    }
                    _ => {
                        // Is this even possible? maybe just panic
                        return Err(CompilerError::InvalidToken {
                            actual: self.current_kind(),
                            line: self.current_line(),
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
            match self.current_kind() {
                TokenKind::True => {
                    self.emit_opcode(OpCode::True);
                    previous = current;
                    current = Some(ExpressionKind::Bool);
                }
                TokenKind::False => {
                    self.emit_opcode(OpCode::False);
                    previous = current;
                    current = Some(ExpressionKind::Bool);
                }
                TokenKind::Number => {
                    // todo: replace this method aswell?
                    self.chunk.emit_number(&self.tokens[self.p]);
                    previous = current;
                    current = Some(ExpressionKind::Int);
                }
                TokenKind::String => {
                    self.chunk.emit_string(&self.tokens[self.p]);
                    previous = current;
                    current = Some(ExpressionKind::String);
                }
                TokenKind::Identifier => {
                    self.emit_opcode(OpCode::GetLocal);
                    let mut found = false;
                    // TODO: Function
                    for local in self.locals.last().unwrap() {
                        if local.name == self.tokens[self.p].value {
                            // TODO
                            previous = current;
                            current = Some(local.kind);
                            found = true;
                            self.emit_u8(local.stack_pos as u8);
                            break;
                        }
                    }
                    if !found {
                        return Err(CompilerError::MissingLocal {
                            // Todo: self.current_value() ?
                            name: self.tokens[self.p].value.to_string(),
                            line: self.current_line(),
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
                    return Err(CompilerError::InvalidToken {
                        actual: self.current_kind(),
                        line: self.current_line(),
                    })
                }
            }

            // adding operators after
            match operator {
                Some(Operator::Modulo)
                | Some(Operator::Greater)
                | Some(Operator::GreaterEqual)
                | Some(Operator::Less)
                | Some(Operator::LessEqual)
                | Some(Operator::Subtract)
                | Some(Operator::Multiply)
                | Some(Operator::Divide) => {
                    self.handle_int_int_operator(operator, previous, current)?;
                }
                Some(Operator::BangEqual) => {
                    if &previous != &current {
                        return Err(CompilerError::ComparisonType {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.current_line(),
                        });
                    }
                    match &current {
                        Some(ExpressionKind::String) => self.emit_opcode(OpCode::CompareStringNot),
                        Some(ExpressionKind::Bool) => self.emit_opcode(OpCode::CompareBoolNot),
                        Some(ExpressionKind::Int) => self.emit_opcode(OpCode::CompareIntNot),
                        None => unreachable!("Cant compare none"),
                    }
                    current = Some(ExpressionKind::Bool);
                }

                Some(Operator::EqualEqual) => {
                    if &previous != &current {
                        return Err(CompilerError::ComparisonType {
                            first: previous.unwrap(),
                            second: current.unwrap(),
                            line: self.tokens[self.p].line,
                        });
                    }
                    match &current {
                        Some(ExpressionKind::String) => self.emit_opcode(OpCode::CompareString),
                        Some(ExpressionKind::Bool) => self.emit_opcode(OpCode::CompareBool),
                        Some(ExpressionKind::Int) => self.emit_opcode(OpCode::CompareInt),
                        None => unreachable!("Cant compare none"),
                    }
                    current = Some(ExpressionKind::Bool);
                }

                Some(Operator::Add) => {
                    current = Some(self.handle_add_operator(previous.unwrap(), current.unwrap())?);
                }
                Some(Operator::And) => match (&previous, &current) {
                    (Some(ExpressionKind::Bool), Some(ExpressionKind::Bool)) => {
                        self.emit_opcode(OpCode::And);
                    }
                    _ => return Err(CompilerError::BooleanExpression(self.current_line())),
                },
                Some(Operator::Or) => match (&previous, &current) {
                    (Some(ExpressionKind::Bool), Some(ExpressionKind::Bool)) => {
                        self.emit_opcode(OpCode::Or);
                    }
                    _ => return Err(CompilerError::BooleanExpression(self.current_line())),
                },
                None => {}
            }
            operator = None;
            self.p += 1;
        }
        Ok(current.unwrap())
    }
    fn emit_opcode(&mut self, opcode: OpCode) {
        self.chunk.emit_code(opcode as u8, self.current_line());
    }
    fn emit_u8(&mut self, b: u8) {
        self.chunk.emit_code(b, self.current_line());
    }

    fn handle_int_int_operator(
        &mut self,
        operator: Option<Operator>,
        previous: Option<ExpressionKind>,
        current: Option<ExpressionKind>,
    ) -> Result<()> {
        if &previous != &Some(ExpressionKind::Int) || &current != &Some(ExpressionKind::Int) {
            return Err(CompilerError::NumberOperator {
                operator: Operator::Modulo,
                first: previous.unwrap(),
                second: current.unwrap(),
            });
        }
        let opcode = operator_to_opcode(operator.unwrap());
        self.emit_opcode(opcode);
        Ok(())
    }

    fn handle_add_operator(
        &mut self,
        exp1: ExpressionKind,
        exp2: ExpressionKind,
    ) -> Result<ExpressionKind> {
        let opcode = match (exp1, exp2) {
            (ExpressionKind::String, ExpressionKind::String) => OpCode::StringStringConcat,
            (ExpressionKind::String, ExpressionKind::Int) => OpCode::StringIntConcat,
            (ExpressionKind::Int, ExpressionKind::String) => OpCode::IntStringConcat,
            (ExpressionKind::String, ExpressionKind::Bool) => OpCode::StringBoolConcat,
            (ExpressionKind::Int, ExpressionKind::Int) => OpCode::Add,
            (ExpressionKind::Bool, ExpressionKind::String) => OpCode::BoolStringConcat,
            _ => {
                return Err(CompilerError::InvalidOperatorTypes {
                    first: exp1,
                    second: exp2,
                    line: self.current_line(),
                });
            }
        };
        self.emit_opcode(opcode);
        if exp1 == ExpressionKind::Int && exp2 == ExpressionKind::Int {
            return Ok(ExpressionKind::Int);
        }
        return Ok(ExpressionKind::String);
    }

    fn current_line(&self) -> usize {
        return self.tokens[self.p].line;
    }
    fn current_kind(&self) -> TokenKind {
        return self.tokens[self.p].kind;
    }
}

fn operator_to_opcode(operator: Operator) -> OpCode {
    match operator {
        // TODO: What to do with add?
        //Add => OpCode::Add,
        Operator::Subtract => OpCode::Subtract,
        Operator::Divide => OpCode::Divide,
        Operator::Multiply => OpCode::Multiply,
        Operator::Modulo => OpCode::Modulo,
        Operator::Greater => OpCode::Greater,
        Operator::GreaterEqual => OpCode::GreaterEqual,
        Operator::Less => OpCode::Less,
        Operator::LessEqual => OpCode::LessEqual,
        _ => panic!("Should not be possible"),
    }
}

#[derive(Debug)]
struct Local {
    kind: ExpressionKind,
    name: String,
    stack_pos: usize,
}

// TODO: derive display instead

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
