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
    println!("chunk: {:#?}", chunk);

    while ip < chunk.code.len() {
        let curr_instruction: OpCode = unsafe { std::mem::transmute(chunk.code[ip]) };
        // println!("===============================");
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
                println!("GET LOCAL");
                stack.push(StackValue {
                    i: unsafe { stack[chunk.code[ip] as usize].i },
                })
            }
            OpCode::SetLocal => {
                ip += 1;
                let slot = chunk.code[ip] as usize;
                unsafe { stack[slot].i = stack.last().unwrap().i };
            }
            OpCode::True => {
                stack.push(StackValue { b: true });
            }
            OpCode::False => {
                stack.push(StackValue { b: false });
            }
            OpCode::SetJump => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
            }
            OpCode::JumpIfFalse => {
                let jump_distance = unsafe { stack.pop().unwrap().u };
                let bool = unsafe { stack.pop().unwrap().b };
                println!("JumpIfFalse, bool: {}, jump_distance: {}", bool, jump_distance);
                if !bool {
                    println!("JUMPOING");
                    ip += jump_distance as usize;
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
