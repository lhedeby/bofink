use std::fmt::Display;
use std::io::Write;

use crate::compiler::Chunk;
use crate::opcode::OpCode;

// rust analyzer - cant dervie debug from unions
union StackValue {
    b: bool,
    i: i64,
    u: u8,
}

impl Display for StackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "i: {}, u: {}, bool: {}",
            unsafe { self.i },
            unsafe { self.u },
            unsafe { self.b }
        )
    }
}

struct Vm {
    stack: Vec<StackValue>,
    // TODO: (usize, usize) ? with the second value being the row
    call_stack: Vec<(usize, usize)>,
    stack_offset: usize,
    offsets: Vec<usize>,
    instances: Vec<RuntimeInstance>,
}

struct RuntimeInstance {
    // class: u8,
    values: Vec<StackValue>,
}

impl Display for RuntimeInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Struct RuntimeInstance - values")?;
        for v in &self.values {
            write!(
                f,
                "i: {}, u: {}, bool: {}",
                unsafe { v.i },
                unsafe { v.u },
                unsafe { v.b }
            )?;
        }
        Ok(())
    }
}

fn print_stack(stack: &Vec<StackValue>) {
    println!("Stack:");
    for v in stack {
        println!(
            "i: {}, u: {}, bool: {}",
            unsafe { v.i },
            unsafe { v.u },
            unsafe { v.b }
        );
    }
}

pub fn start(chunk: Chunk, out: &mut impl Write) {
    let mut vm = Vm {
        stack: vec![],
        call_stack: vec![],
        stack_offset: 0,
        offsets: vec![0],
        instances: vec![],
    };
    vm.interpret(chunk, out);
}

impl Vm {
    pub fn interpret(&mut self, mut chunk: Chunk, out: &mut impl Write) {
        let debug_loggin = false;

        let mut ip: usize = 0;
        let mut curr_code = &chunk.code[0];

        while ip < curr_code.len() {
            let curr_instruction: OpCode = unsafe { std::mem::transmute(curr_code[ip]) };

            if debug_loggin {
                println!("===============================");
                println!("curr: '{:?}'", curr_instruction);
                for b in curr_code {
                    print!("{:02x?} ", b);
                }
                println!();
                println!("{:indent$}{}", "", "|", indent = ip * 3);
                println!("{:indent$}{}", "", "|", indent = ip * 3);
                println!(
                    "{:indent$}{} {:?}",
                    "",
                    "|",
                    curr_instruction,
                    indent = ip * 3
                );
                print_stack(&self.stack);
                println!("===============================");
                println!();
            }

            match curr_instruction {
                OpCode::Print => {
                    let val = self.stack.pop().unwrap();
                    writeln!(out, "{}", chunk.strings[unsafe { val.u } as usize])
                        .expect("Unable to write to output");
                }
                OpCode::String => {
                    ip += 1;
                    self.stack.push(StackValue { u: curr_code[ip] });
                }
                OpCode::Modulo => {
                    let num1 = unsafe { self.stack.pop().unwrap().i };
                    let num2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: num2 % num1 });
                }
                OpCode::Add => {
                    let num1 = unsafe { self.stack.pop().unwrap().i };
                    let num2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: num2 + num1 });
                }
                OpCode::Subtract => {
                    let num1 = unsafe { self.stack.pop().unwrap().i };
                    let num2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: num2 - num1 });
                }
                OpCode::Divide => {
                    let num1 = unsafe { self.stack.pop().unwrap().i };
                    let num2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: num2 / num1 });
                }
                OpCode::Multiply => {
                    let num1 = unsafe { self.stack.pop().unwrap().i };
                    let num2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: num2 * num1 });
                }
                OpCode::Negate => {
                    let num = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { i: -num });
                }
                OpCode::Not => {
                    let bool = unsafe { self.stack.pop().unwrap().b };
                    self.stack.push(StackValue { b: !bool });
                }
                OpCode::StringStringConcat => {
                    let s1 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];
                    let s2 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];
                    let ptr = chunk.strings.len() as u8;
                    chunk.strings.push(s2.to_string() + s1);
                    self.stack.push(StackValue { u: ptr });
                }
                OpCode::BoolStringConcat => {
                    let s1 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];
                    let s2 = match unsafe { self.stack.pop().unwrap().b } == true {
                        true => "true",
                        false => "false",
                    };

                    let ptr = chunk.strings.len() as u8;
                    chunk.strings.push(s2.to_string() + &s1);
                    self.stack.push(StackValue { u: ptr });
                }
                OpCode::StringBoolConcat => {
                    // TODO: use match
                    let s1 = if unsafe { self.stack.pop().unwrap().b } == true {
                        "true"
                    } else {
                        "false"
                    };
                    let s2 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];

                    let ptr = chunk.strings.len() as u8;
                    chunk.strings.push(s2.to_string() + &s1);
                    self.stack.push(StackValue { u: ptr });
                }
                OpCode::IntStringConcat => {
                    let s1 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];
                    let s2 = unsafe { self.stack.pop().unwrap().i };

                    let ptr = chunk.strings.len() as u8;
                    chunk.strings.push(s2.to_string() + &s1);
                    self.stack.push(StackValue { u: ptr });
                }
                OpCode::StringIntConcat => {
                    let s1 = unsafe { self.stack.pop().unwrap().i.to_string() };
                    let s2 = &chunk.strings[unsafe { self.stack.pop().unwrap().u } as usize];

                    let ptr = chunk.strings.len() as u8;
                    chunk.strings.push(s2.to_string() + &s1);
                    self.stack.push(StackValue { u: ptr });
                }
                OpCode::Int => {
                    ip += 1;
                    self.stack.push(StackValue {
                        i: chunk.ints[curr_code[ip] as usize],
                    });
                }
                OpCode::GetLocal => {
                    ip += 1;
                    self.stack.push(StackValue {
                        i: unsafe { self.stack[(curr_code[ip] as usize) + self.stack_offset].i },
                    })
                }
                OpCode::SetLocal => {
                    ip += 1;
                    let slot = curr_code[ip] as usize;
                    unsafe { self.stack[slot].i = self.stack.pop().unwrap().i };
                }
                OpCode::True => {
                    self.stack.push(StackValue { b: true });
                }
                OpCode::False => {
                    self.stack.push(StackValue { b: false });
                }
                OpCode::And => {
                    let v1 = unsafe { self.stack.pop().unwrap().b };
                    let v2 = unsafe { self.stack.pop().unwrap().b };
                    self.stack.push(StackValue { b: v1 && v2 })
                }
                OpCode::Or => {
                    let v1 = unsafe { self.stack.pop().unwrap().b };
                    let v2 = unsafe { self.stack.pop().unwrap().b };
                    self.stack.push(StackValue { b: v1 || v2 })
                }
                OpCode::CompareInt => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v1 == v2 });
                }
                OpCode::CompareIntNot => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v1 != v2 })
                }
                OpCode::CompareString => {
                    let v1 = unsafe { self.stack.pop().unwrap().u };
                    let v2 = unsafe { self.stack.pop().unwrap().u };
                    self.stack.push(StackValue {
                        b: &chunk.strings[v1 as usize] == &chunk.strings[v2 as usize],
                    })
                }
                OpCode::CompareStringNot => {
                    let v1 = unsafe { self.stack.pop().unwrap().u };
                    let v2 = unsafe { self.stack.pop().unwrap().u };
                    self.stack.push(StackValue {
                        b: &chunk.strings[v1 as usize] != &chunk.strings[v2 as usize],
                    })
                }
                OpCode::CompareBool => {
                    let v1 = unsafe { self.stack.pop().unwrap().b };
                    let v2 = unsafe { self.stack.pop().unwrap().b };
                    self.stack.push(StackValue { b: v1 == v2 })
                }
                OpCode::CompareBoolNot => {
                    let v1 = unsafe { self.stack.pop().unwrap().b };
                    let v2 = unsafe { self.stack.pop().unwrap().b };
                    self.stack.push(StackValue { b: v1 != v2 })
                }
                OpCode::Less => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v2 < v1 })
                }
                OpCode::LessEqual => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v2 <= v1 })
                }
                OpCode::Greater => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v2 > v1 })
                }
                OpCode::GreaterEqual => {
                    let v1 = unsafe { self.stack.pop().unwrap().i };
                    let v2 = unsafe { self.stack.pop().unwrap().i };
                    self.stack.push(StackValue { b: v2 >= v1 })
                }
                OpCode::SetJump => {
                    ip += 1;
                    self.stack.push(StackValue { u: curr_code[ip] });
                }
                OpCode::JumpIfFalse => {
                    let jump_distance = unsafe { self.stack.pop().unwrap().u };

                    let bool = unsafe { self.stack.pop().unwrap().b };
                    if !bool {
                        ip += jump_distance as usize;
                    }
                }
                OpCode::JumpBack => {
                    let jump_distance = unsafe { self.stack.pop().unwrap().u };
                    ip -= jump_distance as usize;
                }
                OpCode::FunctionCall => {
                    ip += 1;
                    let func_idx = curr_code[ip] as usize;
                    let return_func = match self.call_stack.last() {
                        Some(_) => func_idx,
                        None => 0,
                    };
                    self.call_stack.push((ip + 1, return_func));
                    curr_code = &chunk.code[func_idx];
                    ip = 0;
                    continue;
                }
                OpCode::PopStack => {
                    self.stack.pop();
                }
                OpCode::SetOffset => {
                    ip += 1;
                    let vars_in_current_scope = curr_code[ip];
                    self.stack_offset = self.stack.len() - vars_in_current_scope as usize;
                    self.offsets.push(self.stack_offset);
                }
                OpCode::PopOffset => {
                    self.offsets.pop();
                    self.stack_offset = self.offsets.last().unwrap().clone();
                }
                OpCode::Return => {
                    let call_frame = self.call_stack.pop().unwrap();
                    for _ in 0..curr_code[ip + 1] {
                        self.stack.pop();
                    }
                    curr_code = &chunk.code[call_frame.1];
                    ip = call_frame.0;
                    continue;
                }
                OpCode::ReturnValue => {
                    let return_position = self.call_stack.pop().unwrap();
                    let return_value = self.stack.pop().unwrap();
                    for _ in 0..curr_code[ip + 1] {
                        self.stack.pop();
                    }
                    curr_code = &chunk.code[return_position.1];
                    ip = return_position.0;
                    self.stack.push(return_value);

                    continue;
                }
                OpCode::CreateInstance => {
                    let mut instance = RuntimeInstance { values: vec![] };
                    ip += 1;
                    let field_count = curr_code[ip];
                    let mut temp: Vec<StackValue> = vec![];
                    for _ in 0..field_count {
                        let val = self.stack.pop().unwrap();
                        temp.push(val);
                    }
                    while temp.len() > 0 {
                        instance.values.push(temp.pop().unwrap());
                    }
                    self.stack.push(StackValue {
                        u: self.instances.len() as u8,
                    });
                    self.instances.push(instance);
                }
                OpCode::GetField => {
                    ip += 1;
                    let instance_idx = unsafe { self.stack.pop().unwrap().u as usize };
                    let field_idx = curr_code[ip] as usize;

                    self.stack.push(StackValue {
                        i: unsafe { self.instances[instance_idx].values[field_idx].i },
                    });
                }
                OpCode::SetField => {
                    let new_value = unsafe { self.stack.pop().unwrap().i };
                    let mut instance_idx = unsafe { self.stack.pop().unwrap().u as usize };
                    ip += 1;

                    let field_levels = curr_code[ip] as usize;
                    for i in 0..field_levels {
                        ip += 1;
                        let temp = curr_code[ip] as usize;
                        if i == (field_levels - 1) {
                            self.instances[instance_idx].values[temp] = StackValue { i: new_value };
                        } else {
                            // TODO: works with u - why?
                            instance_idx =
                                unsafe { self.instances[instance_idx].values[temp].u as usize };
                        }
                    }
                }
                _ => panic!(
                    "No implementation for instruction '{:#?}'",
                    curr_instruction
                ),
            }
            ip += 1;
        }
    }
}
