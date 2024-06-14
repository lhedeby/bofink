use std::io::Write;

use crate::compiler::Chunk;
use crate::opcode::OpCode;

// rust analyzer - cant dervie debug from unions
union StackValue {
    b: bool,
    i: i64,
    u: u8,
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

pub fn interpret(mut chunk: Chunk, out: &mut impl Write) {
    let mut ip: usize = 0;
    let mut stack: Vec<StackValue> = vec![];
    let mut call_stack: Vec<usize> = vec![];
    let mut stack_offset = 0;
    let mut offsets: Vec<usize> = vec![0];

    let debug_loggin = false;

    while ip < chunk.code.len() {
        let curr_instruction: OpCode = unsafe { std::mem::transmute(chunk.code[ip]) };

        if debug_loggin {
            println!("===============================");
            println!("curr: '{:?}'", curr_instruction);
            for b in &chunk.code {
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
            print_stack(&stack);
            println!("===============================");
            println!();
        }

        match curr_instruction {
            OpCode::Print => {
                let val = stack.pop().unwrap();
                writeln!(out, "{}", chunk.strings[unsafe { val.u } as usize])
                    .expect("Unable to write to output");
            }
            OpCode::String => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
            }
            OpCode::Modulo => {
                let num1 = unsafe { stack.pop().unwrap().i };
                let num2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: num2 % num1 });
            }
            OpCode::Add => {
                let num1 = unsafe { stack.pop().unwrap().i };
                let num2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: num2 + num1 });
            }
            OpCode::Subtract => {
                let num1 = unsafe { stack.pop().unwrap().i };
                let num2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: num2 - num1 });
            }
            OpCode::Divide => {
                let num1 = unsafe { stack.pop().unwrap().i };
                let num2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: num2 / num1 });
            }
            OpCode::Multiply => {
                let num1 = unsafe { stack.pop().unwrap().i };
                let num2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: num2 * num1 });
            }
            OpCode::Negate => {
                let num = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { i: -num });
            }
            OpCode::Not => {
                let bool = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: !bool });
            }
            OpCode::StringStringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::BoolStringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = if unsafe { stack.pop().unwrap().b } == true {
                    "true"
                } else {
                    "false"
                };

                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + &s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::StringBoolConcat => {
                let s1 = if unsafe { stack.pop().unwrap().b } == true {
                    "true"
                } else {
                    "false"
                };
                let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];

                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + &s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::IntStringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = unsafe { stack.pop().unwrap().i };

                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + &s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::StringIntConcat => {
                let s1 = unsafe { stack.pop().unwrap().i.to_string() };
                let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];

                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + &s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::Int => {
                ip += 1;
                stack.push(StackValue {
                    i: chunk.ints[chunk.code[ip] as usize],
                });
            }
            OpCode::GetLocal => {
                ip += 1;
                stack.push(StackValue {
                    i: unsafe { stack[(chunk.code[ip] as usize) + stack_offset].i },
                })
            }
            OpCode::SetLocal => {
                ip += 1;
                let slot = chunk.code[ip] as usize;
                unsafe { stack[slot].i = stack.pop().unwrap().i };
            }
            OpCode::True => {
                stack.push(StackValue { b: true });
            }
            OpCode::False => {
                stack.push(StackValue { b: false });
            }
            OpCode::And => {
                let v1 = unsafe { stack.pop().unwrap().b };
                let v2 = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: v1 && v2 })
            }
            OpCode::Or => {
                let v1 = unsafe { stack.pop().unwrap().b };
                let v2 = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: v1 || v2 })
            }
            OpCode::CompareInt => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push( StackValue { b: v1 == v2 });
            }
            OpCode::CompareIntNot => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v1 != v2 })
            }
            OpCode::CompareString => {
                let v1 = unsafe { stack.pop().unwrap().u };
                let v2 = unsafe { stack.pop().unwrap().u };
                stack.push(StackValue {
                    b: &chunk.strings[v1 as usize] == &chunk.strings[v2 as usize],
                })
            }
            OpCode::CompareStringNot => {
                let v1 = unsafe { stack.pop().unwrap().u };
                let v2 = unsafe { stack.pop().unwrap().u };
                stack.push(StackValue {
                    b: &chunk.strings[v1 as usize] != &chunk.strings[v2 as usize],
                })
            }
            OpCode::CompareBool => {
                let v1 = unsafe { stack.pop().unwrap().b };
                let v2 = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: v1 == v2 })
            }
            OpCode::CompareBoolNot => {
                let v1 = unsafe { stack.pop().unwrap().b };
                let v2 = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: v1 != v2 })
            }
            OpCode::Less => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v2 < v1 })
            }
            OpCode::LessEqual => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v2 <= v1 })
            }
            OpCode::Greater => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v2 > v1 })
            }
            OpCode::GreaterEqual => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v2 >= v1 })
            }
            OpCode::SetJump => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
            }
            OpCode::JumpIfFalse => {
                let jump_distance = unsafe { stack.pop().unwrap().u };
                let bool = unsafe { stack.pop().unwrap().b };
                if !bool {
                    ip += jump_distance as usize;
                }
            }
            OpCode::JumpForward => {
                let jump_distance = unsafe { stack.pop().unwrap().u };
                ip += jump_distance as usize;
            }
            OpCode::JumpBack => {
                let jump_distance = unsafe { stack.pop().unwrap().u };
                ip -= jump_distance as usize;
            }
            OpCode::FunctionCall => {
                ip += 1;
                let jump_position = chunk.code[ip];
                call_stack.push(ip + 1);
                ip = chunk.funcs[jump_position as usize];
                continue;
            }
            OpCode::PopStack => {
                stack.pop();
            }
            OpCode::SetOffset => {
                ip += 1;
                let vars_in_current_scope = chunk.code[ip];
                stack_offset = stack.len() - vars_in_current_scope as usize;
                offsets.push(stack_offset);
            }
            OpCode::PopOffset => {
                offsets.pop();
                stack_offset = offsets.last().unwrap().clone();
            }
            OpCode::Return => {
                let return_position = call_stack.pop().unwrap();
                for _ in 0..chunk.code[ip + 1] {
                    stack.pop();
                }
                ip = return_position;
                continue;
            }
            OpCode::ReturnValue => {
                let return_position = call_stack.pop().unwrap();

                let return_value = stack.pop().unwrap();
                for _ in 0..chunk.code[ip + 1] {
                    stack.pop();
                }
                ip = return_position;
                stack.push(return_value);

                continue;
            }
            _ => panic!(
                "No implementation for instruction '{:#?}'",
                curr_instruction
            ),
        }
        ip += 1;
    }
}
