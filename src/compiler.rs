use std::collections::HashMap;

use crate::enums::{CompilerError, ExpressionKind, TokenKind};

use crate::opcode::OpCode;
use crate::scanner::{Scanner, Token};

struct Compiler {
    chunk: Chunk,
    p: usize,
    locals: Vec<Vec<Local>>,
    local_count: usize,
    tokens: Vec<Token>,
    functions: HashMap<String, Function>,
    function_return_kind: Option<ExpressionKind>,
    scopes: Vec<usize>,
}

type Result<T> = std::result::Result<T, CompilerError>;

pub fn compile(source: String) -> Result<Chunk> {
    let mut compiler = Compiler {
        chunk: Chunk {
            curr_func: 0,
            code: vec![vec![]],
            line: vec![],
            strings: vec![],
            funcs: vec![],
            ints: vec![],
            patch_list: vec![],
            runtime_functions: vec![],
        },
        p: 0,
        locals: vec![vec![]],
        functions: HashMap::new(),
        function_return_kind: None,
        local_count: 0,
        scopes: vec![],
        tokens: Scanner::get_tokens(source.clone()),
    };

    println!("{:?}", compiler.chunk);
    match compiler.declaration() {
        Ok(_) => Ok(compiler.chunk),
        Err(e) => {
            let line_index = compiler.current_line();
            let line = source.lines().nth(line_index - 1).unwrap();
            println!("{}", e);
            println!("{}   _________", " ".repeat(line_index.to_string().len()));
            println!("{}  |", " ".repeat(line_index.to_string().len()));
            println!("{}  | {}", line_index, line);
            println!("{}  |_________\n", " ".repeat(line_index.to_string().len()));
            Err(e)
        }
    }
}

impl Compiler {
    fn check_expression_kind(
        &mut self,
        kind: ExpressionKind,
        expected: ExpressionKind,
    ) -> Result<()> {
        if kind != expected {
            return Err(CompilerError::Type {
                actual: kind,
                expected,
                line: self.current_line(),
            });
        }
        Ok(())
    }

    /// Compiles an `expression` to bytecode.
    fn expression(&mut self) -> Result<ExpressionKind> {
        return self.or();
    }

    fn or(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.and()?;
        while self.current_kind() == TokenKind::Or {
            self.check_expression_kind(left_kind, ExpressionKind::Bool)?;
            self.p += 1;
            let right_kind = self.and()?;
            self.check_expression_kind(right_kind, ExpressionKind::Bool)?;
            self.emit_opcode(OpCode::Or);
            return Ok(ExpressionKind::Bool);
        }
        Ok(left_kind)
    }

    fn and(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.equality()?;
        while self.current_kind() == TokenKind::And {
            self.check_expression_kind(left_kind, ExpressionKind::Bool)?;
            self.p += 1;
            let right_kind = self.equality()?;
            self.check_expression_kind(right_kind, ExpressionKind::Bool)?;
            self.emit_opcode(OpCode::And);
            return Ok(ExpressionKind::Bool);
        }
        Ok(left_kind)
    }

    fn equality(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.comparison()?;
        let mut return_type = left_kind;
        while self.current_kind() == TokenKind::BangEqual
            || self.current_kind() == TokenKind::EqualEqual
        {
            let token_kind = self.current_kind();
            self.p += 1;
            let right_kind = self.comparison()?;
            if left_kind != right_kind {
                return Err(CompilerError::ComparisonType {
                    first: left_kind,
                    second: right_kind,
                    line: self.current_line(),
                });
            }
            match token_kind {
                TokenKind::BangEqual => match left_kind {
                    ExpressionKind::Bool => self.emit_opcode(OpCode::CompareBoolNot),
                    ExpressionKind::String => self.emit_opcode(OpCode::CompareStringNot),
                    ExpressionKind::Int => self.emit_opcode(OpCode::CompareIntNot),
                    ExpressionKind::None => {
                        return Err(CompilerError::NoneValue {
                            line: self.current_line(),
                        })
                    }
                },
                TokenKind::EqualEqual => match left_kind {
                    ExpressionKind::Bool => self.emit_opcode(OpCode::CompareBool),
                    ExpressionKind::String => self.emit_opcode(OpCode::CompareString),
                    ExpressionKind::Int => self.emit_opcode(OpCode::CompareInt),
                    ExpressionKind::None => {
                        return Err(CompilerError::NoneValue {
                            line: self.current_line(),
                        })
                    }
                },
                _ => unreachable!(),
            }
            return_type = ExpressionKind::Bool;
        }
        Ok(return_type)
    }

    fn comparison(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.term()?;
        let mut return_kind = left_kind;
        loop {
            match self.current_kind() {
                TokenKind::Greater
                | TokenKind::GreaterEqual
                | TokenKind::Less
                | TokenKind::LessEqual => {}
                _ => break,
            }
            self.check_expression_kind(left_kind, ExpressionKind::Int)?;
            let token_kind = self.current_kind();
            self.p += 1;
            let right_kind = self.term()?;
            self.check_expression_kind(right_kind, ExpressionKind::Int)?;

            match token_kind {
                TokenKind::Greater => self.emit_opcode(OpCode::Greater),
                TokenKind::GreaterEqual => self.emit_opcode(OpCode::GreaterEqual),
                TokenKind::Less => self.emit_opcode(OpCode::Less),
                TokenKind::LessEqual => self.emit_opcode(OpCode::LessEqual),
                _ => unreachable!(),
            }
            return_kind = ExpressionKind::Bool;
        }
        Ok(return_kind)
    }

    fn term(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.factor()?;
        let mut return_kind = left_kind;
        loop {
            let token_kind = self.current_kind();
            match token_kind {
                TokenKind::Minus | TokenKind::Plus => {}
                _ => break,
            }
            self.p += 1;
            let right_kind = self.factor()?;

            match token_kind {
                TokenKind::Minus => {
                    self.check_expression_kind(left_kind, ExpressionKind::Int)?;
                    self.check_expression_kind(right_kind, ExpressionKind::Int)?;
                    self.emit_opcode(OpCode::Subtract)
                }
                TokenKind::Plus => match (left_kind, right_kind) {
                    (ExpressionKind::Bool, ExpressionKind::String) => {
                        self.emit_opcode(OpCode::BoolStringConcat)
                    }
                    (ExpressionKind::String, ExpressionKind::Bool) => {
                        self.emit_opcode(OpCode::StringBoolConcat)
                    }
                    (ExpressionKind::Int, ExpressionKind::String) => {
                        self.emit_opcode(OpCode::IntStringConcat)
                    }
                    (ExpressionKind::String, ExpressionKind::Int) => {
                        self.emit_opcode(OpCode::StringIntConcat)
                    }
                    (ExpressionKind::String, ExpressionKind::String) => {
                        self.emit_opcode(OpCode::StringStringConcat)
                    }
                    (ExpressionKind::Int, ExpressionKind::Int) => self.emit_opcode(OpCode::Add),
                    _ => todo!("invalid types erro"),
                },
                _ => unreachable!(),
            }
            if left_kind == ExpressionKind::String || right_kind == ExpressionKind::String {
                return_kind = ExpressionKind::String;
            } else {
                return_kind = ExpressionKind::Int;
            }
        }
        Ok(return_kind)
    }

    fn factor(&mut self) -> Result<ExpressionKind> {
        let left_kind = self.unary()?;
        let mut return_kind = left_kind;
        loop {
            match self.current_kind() {
                TokenKind::Slash | TokenKind::Star | TokenKind::Percent => {}
                _ => break,
            }
            self.check_expression_kind(left_kind, ExpressionKind::Int)?;
            let token_kind = self.current_kind();
            self.p += 1;
            let right_kind = self.unary()?;
            self.check_expression_kind(right_kind, ExpressionKind::Int)?;

            match token_kind {
                TokenKind::Slash => self.emit_opcode(OpCode::Divide),
                TokenKind::Star => self.emit_opcode(OpCode::Multiply),
                TokenKind::Percent => self.emit_opcode(OpCode::Modulo),
                _ => unreachable!(),
            }
            return_kind = ExpressionKind::Int;
        }
        Ok(return_kind)
    }

    fn unary(&mut self) -> Result<ExpressionKind> {
        match self.current_kind() {
            TokenKind::Bang => {
                self.p += 1;
                let kind = self.unary()?;
                self.check_expression_kind(kind, ExpressionKind::Bool)?;
                self.emit_opcode(OpCode::Not);
                return Ok(kind);
            }
            TokenKind::Minus => {
                self.p += 1;
                let kind = self.unary()?;
                self.check_expression_kind(kind, ExpressionKind::Int)?;
                self.emit_opcode(OpCode::Negate);
                return Ok(kind);
            }
            _ => {
                return self.primary();
            }
        }
    }

    fn primary(&mut self) -> Result<ExpressionKind> {
        let curr_kind = self.current_kind();
        self.p += 1;
        match curr_kind {
            TokenKind::False => {
                self.emit_opcode(OpCode::False);
                Ok(ExpressionKind::Bool)
            }
            TokenKind::True => {
                self.emit_opcode(OpCode::True);
                Ok(ExpressionKind::Bool)
            }
            TokenKind::Nil => {
                panic!("rd_primary -> Nil")
            }
            TokenKind::Number => {
                self.chunk.emit_number(&self.tokens[self.p - 1]);
                Ok(ExpressionKind::Int)
            }
            TokenKind::String => {
                self.chunk.emit_string(&self.tokens[self.p - 1]);
                Ok(ExpressionKind::String)
            }
            TokenKind::Identifier => {
                let identifier = self.tokens[self.p - 1].value.to_string();
                match self.tokens[self.p].kind {
                    // function call Todo: just dont...
                    TokenKind::LeftParen => Ok(self.function_call(identifier)?.unwrap()),
                    TokenKind::Dot => todo!("not implemented"),
                    _ => self.get_local(),
                }
            }
            TokenKind::LeftParen => {
                // Todo:???
                let kind = self.expression();
                self.consume_token(TokenKind::RightParen)?;
                return kind;
            }
            _ => unreachable!("Not a valid token"),
        }
    }

    fn get_local(&mut self) -> Result<ExpressionKind> {
        let res = match self.locals.last() {
            Some(l_vec) => match l_vec
                .iter()
                .find(|l| l.name == self.tokens[self.p - 1].value)
            {
                Some(l) => (l.stack_pos, l.kind),
                None => {
                    return Err(CompilerError::MissingLocal {
                        name: self.tokens[self.p - 1].value.to_string(),
                        line: self.current_line(),
                    })
                }
            },
            None => unreachable!("Locals Vec should never be empty"),
        };
        self.emit_opcode(OpCode::GetLocal);
        self.emit_u8(res.0 as u8);
        Ok(res.1)
    }

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
            column: token.column,
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
                column: token.column,
                value: token.value.to_string(),
            });
        }
        None
    }

    fn local_declaration(&mut self) -> Result<()> {
        let is_mut = match self.current_kind() {
            TokenKind::Mut => true,
            TokenKind::Let => false,
            _ => unreachable!(),
        };
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

        let type_kind = match self.consume_if_match(TokenKind::Colon) {
            Some(_) => {
                self.p += 1;
                match self.tokens[self.p - 1].kind {
                    TokenKind::Int => ExpressionKind::Int,
                    TokenKind::Bool => ExpressionKind::Bool,
                    TokenKind::String => ExpressionKind::String,
                    _ => {
                        return Err(CompilerError::NotAType {
                            kind: self.tokens[self.p].kind,
                            line: self.current_line(),
                        })
                    }
                }
            }
            None => ExpressionKind::None,
        };
        let consumed_token = self.consume_token(TokenKind::Equal)?;
        let kind = self.expression()?;

        if type_kind != ExpressionKind::None && kind != type_kind {
            return Err(CompilerError::DelcarationType {
                expected: type_kind,
                actual: kind,
                line: consumed_token.line,
            });
        }

        self.add_local(identifier, kind, is_mut);
        self.consume_token(TokenKind::Semicolon)?;
        Ok(())
    }
    fn class_declaration(&mut self) -> Result<()> {
        Ok(())
    }

    fn declaration(&mut self) -> Result<()> {
        loop {
            match self.current_kind() {
                // type should be a first class member?
                // 'typeof' built in function?
                TokenKind::Mut | TokenKind::Let => {
                    self.local_declaration()?;
                }
                TokenKind::Class => {
                    self.class_declaration()?;
                    todo!("class");
                }
                // TokenKind::Str => {
                //     self.local_declaration(ExpressionKind::String)?;
                // }
                // TokenKind::Bool => {
                //     self.local_declaration(ExpressionKind::Bool)?;
                // }
                // TokenKind::Int => {
                //     self.local_declaration(ExpressionKind::Int)?;
                // }
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

                    // self.emit_opcode(OpCode::SetJump);
                    // // todo self placeholder?
                    // self.chunk.emit_placeholder(0);
                    // self.emit_opcode(OpCode::JumpForward);
                    // let fun_start = self.chunk.code.len();
                    // self.chunk.funcs.push(fun_start);

                    // let func_chunk = Chunk {
                    //     code: vec![],
                    //     line: vec![],
                    //     strings: vec![],
                    //     funcs: vec![],
                    //     ints: vec![],
                    //     patch_list: vec![],
                    //     runtime_functions: vec![],
                    // };
                    // let prev_chunk = &self.chunk;
                    // let prev_chunk = std::mem::replace(&mut self.chunk, func_chunk);
                    // self.chunk = func_chunk;
                    // self.chunk = *prev_chunk;
                    //
                    // TODO: this wont work with multiple functions, need a stack or something
                    let old_func = self.chunk.curr_func;
                    // self.chunk.curr_func += 1;
                    self.chunk.curr_func = self.chunk.code.len();
                    self.chunk.code.push(vec![]);

                    let fun_count = self.functions.len();
                    if fun_count >= u8::MAX as usize {
                        return Err(CompilerError::MaxFunctions);
                    }
                    self.p += 1;
                    self.consume_token(TokenKind::LeftParen)?;
                    let mut function = Function {
                        index: fun_count as u8 + 1,
                        params: vec![],
                        return_type: None,
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
                        self.add_local(param_name, param_kind, true);
                        self.consume_if_match(TokenKind::Comma);
                    }

                    self.consume_token(TokenKind::RightParen)?;
                    function.return_type = match self.current_kind() {
                        TokenKind::LeftBrace => None,
                        TokenKind::Int => {
                            self.p += 1;
                            Some(ExpressionKind::Int)
                        }
                        TokenKind::Str => {
                            self.p += 1;
                            Some(ExpressionKind::String)
                        }
                        TokenKind::Bool => {
                            self.p += 1;
                            Some(ExpressionKind::Bool)
                        }
                        _ => panic!("TODO!"),
                    };
                    self.function_return_kind = function.return_type;

                    self.functions.insert(identifier.to_string(), function);
                    self.consume_token(TokenKind::LeftBrace)?;
                    self.declaration()?;

                    self.emit_opcode(OpCode::Return);
                    self.emit_u8(self.local_count as u8);
                    // self.chunk.replace_placeholder();
                    self.locals.pop();
                    self.local_count = self.locals.last().unwrap().len();
                    self.function_return_kind = None;
                    self.chunk.curr_func = old_func;
                    // self.chunk.curr_func -= 1;
                    // self.chunk = prev_chunk;
                    // self.chunk.runtime_functions.push(func_chunk);
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

    fn add_local(&mut self, name: &str, kind: ExpressionKind, is_mut: bool) {
        self.locals.last_mut().unwrap().push(Local {
            name: name.to_string(),
            stack_pos: self.local_count,
            is_mut,
            kind,
        });
        self.local_count += 1;
    }

    //
    // STATEMENTS START
    //

    /// Finds and compiles the correct statement to bytecode.
    fn statement(&mut self) -> Result<()> {
        match self.current_kind() {
            TokenKind::While => self.while_stmt()?,
            TokenKind::If => self.if_stmt()?,
            TokenKind::Print => self.print_stmt()?,
            TokenKind::Return => self.return_stmt()?,
            TokenKind::Identifier => self.identifier_stmt(self.tokens[self.p].value.to_string())?,
            TokenKind::For => self.for_stmt()?,
            // dont know if I should allow arbitrary blocks
            //TokenKind::LeftBrace => {}
            _ => {
                self.expression()?;
            }
        }
        Ok(())
    }

    /// Compiles a `while` statement to bytecode.
    fn while_stmt(&mut self) -> Result<()> {
        let jump_point = self.chunk.code[self.chunk.curr_func].len();
        self.p += 1;
        self.expression()?;
        self.emit_opcode(OpCode::SetJump);
        self.chunk.emit_placeholder(0);
        self.emit_opcode(OpCode::JumpIfFalse);
        self.start_scope()?;
        self.declaration()?;
        self.end_scope();
        self.emit_opcode(OpCode::SetJump);
        self.emit_u8((self.chunk.code[self.chunk.curr_func].len() - jump_point + 2) as u8);
        self.emit_opcode(OpCode::JumpBack);
        self.chunk.replace_placeholder();
        Ok(())
    }

    /// Compiles an `if` statement to bytecode.
    fn if_stmt(&mut self) -> Result<()> {
        self.p += 1;
        self.expression()?;
        self.emit_opcode(OpCode::SetJump);
        self.chunk.emit_placeholder(0);
        self.emit_opcode(OpCode::JumpIfFalse);
        self.start_scope()?;
        self.declaration()?;
        self.end_scope();
        self.chunk.replace_placeholder();
        Ok(())
    }

    /// Compiles a `print` statement to bytecode.
    fn print_stmt(&mut self) -> Result<()> {
        self.p += 1;
        self.expression()?;
        self.emit_opcode(OpCode::Print);
        self.consume_token(TokenKind::Semicolon)?;
        Ok(())
    }

    /// Compiles a `return` statement to bytecode.
    fn return_stmt(&mut self) -> Result<()> {
        self.p += 1;
        let return_type = self.expression()?;
        match self.function_return_kind {
            Some(kind) => {
                if kind != return_type {
                    return Err(CompilerError::Type {
                        actual: return_type,
                        expected: kind,
                        line: self.current_line(),
                    });
                }
                self.emit_opcode(OpCode::ReturnValue);
                self.emit_u8(self.local_count as u8);
            }
            None => {
                if return_type != ExpressionKind::None {
                    return Err(CompilerError::ReturnValueFromVoid {
                        kind: return_type,
                        line: self.current_line(),
                    });
                }
                self.emit_opcode(OpCode::Return);
            }
        }
        self.consume_token(TokenKind::Semicolon)?;
        Ok(())
    }

    /// Compiles an `indentifier` statement to bytecode.
    fn identifier_stmt(&mut self, identifier_name: String) -> Result<()> {
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
                        if !local.is_mut {
                            let error_token = Self::get_error_token(&self.tokens[self.p]);
                            return Err(CompilerError::CantMut { token: error_token });
                        }
                        self.emit_u8(local.stack_pos as u8);
                        break;
                    }
                }
            }
            // function call
            TokenKind::LeftParen => {
                // This kind is never needed?
                let _kind = self.function_call(identifier_name)?;
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
        Ok(())
    }

    /// Compiles a `for` statement to bytecode.
    fn for_stmt(&mut self) -> Result<()> {
        self.p += 1;
        let consumed_token = self.consume_token(TokenKind::Identifier)?;
        let iter_name = &consumed_token.value.to_string();
        self.consume_token(TokenKind::In)?;
        let consumed_token = self.consume_token(TokenKind::Number)?;
        let loop_start = &consumed_token.value.parse::<i64>().unwrap();

        // Emit local
        self.chunk.emit_number(&consumed_token);
        self.add_local(iter_name, ExpressionKind::Int, true);

        let jump_point = self.chunk.code[self.chunk.curr_func].len();

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
            column: 0,
        };
        // todo: self.emit_number?
        self.chunk.emit_number(&dummy_token);
        self.emit_opcode(OpCode::Add);
        self.emit_opcode(OpCode::SetLocal);
        self.emit_u8(iterator_stack_pos);
        self.emit_opcode(OpCode::SetJump);
        self.emit_u8((self.chunk.code[self.chunk.curr_func].len() - jump_point + 2) as u8);
        self.emit_opcode(OpCode::JumpBack);
        self.chunk.replace_placeholder();
        Ok(())
    }

    //
    // STATEMENTS END
    //

    // TODO  handle the case where the function has a return type
    fn function_call(&mut self, identifier_name: String) -> Result<Option<ExpressionKind>> {
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
            self.local_count -= 1;
        }
        self.emit_opcode(OpCode::PopOffset);

        Ok(function.return_type)
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.chunk.emit_code(opcode as u8, self.current_line());
    }
    fn emit_u8(&mut self, b: u8) {
        self.chunk.emit_code(b, self.current_line());
    }

    fn get_error_token(token: &Token) -> Token {
        Token {
            kind: token.kind,
            line: token.line,
            column: token.column,
            value: token.value.to_string(),
        }
    }

    fn current_line(&self) -> usize {
        return self.tokens[self.p].line;
    }
    fn current_kind(&self) -> TokenKind {
        return self.tokens[self.p].kind;
    }
}

#[derive(Debug)]
struct Local {
    kind: ExpressionKind,
    name: String,
    is_mut: bool,
    stack_pos: usize,
}

#[derive(Clone)]
struct Function {
    index: u8,
    params: Vec<Param>,
    return_type: Option<ExpressionKind>,
}
#[derive(Debug)]
pub struct RuntimeFunction {
    chunk: Chunk,
}

struct Class {}

#[derive(Clone)]
struct Param {
    kind: ExpressionKind,
}

#[derive(Debug)]
pub struct Chunk {
    pub curr_func: usize,
    pub code: Vec<Vec<u8>>,
    pub line: Vec<usize>,
    pub strings: Vec<String>,
    pub ints: Vec<i64>,
    pub patch_list: Vec<usize>,
    pub funcs: Vec<usize>,
    pub runtime_functions: Vec<Chunk>,
}

impl Chunk {
    fn emit_placeholder(&mut self, line: usize) {
        self.patch_list.push(self.code[self.curr_func].len());
        self.code[self.curr_func].push(0);
        self.line.push(line);
    }

    fn replace_placeholder(&mut self) {
        if let Some(p) = self.patch_list.pop() {
            let jump_len = self.code[self.curr_func].len() - p - 2;
            self.code[self.curr_func][p] = jump_len as u8;
        } else {
            panic!("Patch list is empty");
        }
    }
    fn emit_code(&mut self, b: u8, line: usize) {
        println!("{:?}", self.code);
        self.code[self.curr_func].push(b);
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
