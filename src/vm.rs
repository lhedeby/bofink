use crate::compiler::Chunk;
use crate::opcode::OpCode;

// rust analyzer - cant dervie debug from unions
union StackValue {
    b: bool,
    i: i64,
    u: u8,
}

fn print_stack(stack: &Vec<StackValue>) {
    println!("=== STACK START === ");
    for v in stack {
        println!(
            "i: {}, u: {}, bool: {}",
            unsafe { v.i },
            unsafe { v.u },
            unsafe { v.b }
        );
    }
    println!("=== STACK END === ");
}

pub fn interpret(mut chunk: Chunk) {
    let mut ip: usize = 0;
    let mut stack: Vec<StackValue> = vec![];

    while ip < chunk.code.len() {
        let curr_instruction: OpCode = unsafe { std::mem::transmute(chunk.code[ip]) };
        println!(
            "=== DEBUG === executing instruction: '{:?}', line: '{}'",
            curr_instruction, chunk.line[ip]
        );
        print_stack(&stack);
        match curr_instruction {
            OpCode::Print => {
                let val = stack.pop().unwrap();
                println!(
                    "===================================== Print Stmt === : {}",
                    chunk.strings[unsafe { val.u } as usize]
                )
            }
            OpCode::String => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
            }
            OpCode::StringStringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::IntStringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = unsafe { stack.pop().unwrap().i };

                // let s1 = unsafe { stack.pop().unwrap().i.to_string() };
                // let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];

                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + &s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::StringIntConcat => {
                // let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                // let s2 = unsafe { stack.pop().unwrap().i };

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
                println!("ip: {ip}");
                println!("{:?}", chunk.code);
                println!("ins: {:?}", chunk.code[ip]);
                stack.push(StackValue {
                    i: unsafe { stack[chunk.code[ip] as usize].i },
                })
            }
            OpCode::SetLocal => {
                ip += 1;
                let slot = chunk.code[ip] as usize;
                unsafe { stack[slot].u = stack.last().unwrap().u };
            }
            _ => panic!(
                "No implementation for instruction '{:#?}'",
                curr_instruction
            ),
        }
        ip += 1;
    }
    // println!("chunk: {:#?}", chunk);
}
