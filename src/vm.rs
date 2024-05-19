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

pub fn interpret(mut chunk: Chunk) {
    let mut ip: usize = 0;
    let mut stack: Vec<StackValue> = vec![];
    let mut call_stack: Vec<usize> = vec![];
    let mut stack_offset = 0;
    let mut offsets: Vec<usize> = vec![0];
    println!("chunk: {:#?}", chunk);

    while ip < chunk.code.len() {
        let curr_instruction: OpCode = unsafe { std::mem::transmute(chunk.code[ip]) };
        // println!("===============================");
        // println!("curr: '{:?}'", curr_instruction);
        // for b in &chunk.code {
        //     print!("{:02x?} ", b);
        // }
        // println!();
        // println!("{:indent$}{}", "", "|", indent=ip * 3);
        // println!("{:indent$}{}", "", "|", indent=ip * 3);
        // println!("{:indent$}{} {:?}", "", "|", curr_instruction, indent=ip * 3);
        // print_stack(&stack);
        // println!("===============================");
        // println!();

        match curr_instruction {
            OpCode::Print => {
                let val = stack.pop().unwrap();
                println!("{}", chunk.strings[unsafe { val.u } as usize])
            }
            OpCode::String => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
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
            OpCode::CompareString | OpCode::CompareInt => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v1 == v2 })
            }
            OpCode::CompareBool => {
                let v1 = unsafe { stack.pop().unwrap().b };
                let v2 = unsafe { stack.pop().unwrap().b };
                stack.push(StackValue { b: v1 == v2 })
            }
            OpCode::CompareStringNot | OpCode::CompareIntNot => {
                let v1 = unsafe { stack.pop().unwrap().i };
                let v2 = unsafe { stack.pop().unwrap().i };
                stack.push(StackValue { b: v1 != v2 })
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
                call_stack.push(ip+1);
                ip = chunk.funcs[jump_position as usize];
                continue;
            }
            OpCode::PopStack => {
                stack.pop();
            }
            OpCode::SetOffset => {
                offsets.push(stack.len());
                stack_offset = stack.len();
            }
            OpCode::PopOffset => {
                offsets.pop();
                stack_offset = offsets.last().unwrap().clone();
            }
            OpCode::Return => {
                let return_position = call_stack.pop().unwrap();
                ip = return_position;
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
