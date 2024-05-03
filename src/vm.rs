use crate::compiler::Chunk;
use crate::opcode::OpCode;

union StackValue {
    b: bool,
    i: i64,
    u: u8,
}

pub fn interpret(mut chunk: Chunk) {
    let mut ip: usize = 0;
    let mut stack: Vec<StackValue> = vec![];

    while ip < chunk.code.len() {
        let curr_instruction: OpCode = unsafe { std::mem::transmute(chunk.code[ip]) };
        println!("=== DEBUG === executing instruction: {:?}", curr_instruction);
        match curr_instruction {
            OpCode::Print => {
                let val = stack.pop().unwrap();
                println!(
                    "=== Print Stmt === : {}",
                    chunk.strings[unsafe { val.u } as usize]
                )
            }
            OpCode::String => {
                ip += 1;
                stack.push(StackValue { u: chunk.code[ip] });
            }
            OpCode::StringConcat => {
                let s1 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let s2 = &chunk.strings[unsafe { stack.pop().unwrap().u } as usize];
                let ptr = chunk.strings.len() as u8;
                chunk.strings.push(s2.to_string() + s1);
                stack.push(StackValue { u: ptr });
            }
            OpCode::GetLocal => {
                ip += 1;
                stack.push(StackValue {
                    u: unsafe { stack[chunk.code[ip] as usize].u },
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
