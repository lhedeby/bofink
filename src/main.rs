use std::{
    env, fs,
    io::{stdin, stdout, BufRead, Write},
};

mod compiler;
mod opcode;
mod scanner;
mod vm;

fn repl() {
    //REMOVE THIS
    let mut stdin = stdin().lock();
    let mut buf = String::new();
    // let mut vm = Vm::new();
    loop {
        print!("> ");
        let _ = stdout().flush();
        buf.clear();
        match stdin.read_line(&mut buf) {
            // Ok(_) => _ = vm.interpret(buf.clone()),
            Ok(_) => {
                let chunk = compiler::compile(buf.clone());
                println!("chunk: {:#?}", chunk);
                vm::interpret(chunk);
            }
            Err(_) => break,
        }
    }
}
fn run_file(file_path: &str) {
    // let mut vm = Vm::new();
    match fs::read_to_string(file_path) {
        // Ok(file) => _ = vm.interpret(file),
        Ok(source) => {
            let chunk = compiler::compile(source);
            println!("chunk: {:#?}", chunk);
            vm::interpret(chunk);
        }
        Err(_) => panic!("Error reading file"),
    }
}

fn main() {
    println!("Bofink compiler started...");
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(args.last().unwrap()),
        _ => panic!("Unacceptable usage"),
    }
    // let mut vm = Vm::new();
    // let chunk = Chunk {
    //     code: vec![],
    //     lines: vec![],
    //     values: vec![],
    // };
    // let _res = vm.interpret(chunk);
    println!("Bofink compiler stopped...");
}
