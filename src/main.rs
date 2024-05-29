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
    // let mut stdin = stdin().lock();
    // let mut buf = String::new();
    // // let mut vm = Vm::new();
    // loop {
    //     print!("> ");
    //     let _ = stdout().flush();
    //     buf.clear();
    //     match stdin.read_line(&mut buf) {
    //         // Ok(_) => _ = vm.interpret(buf.clone()),
    //         Ok(_) => {
    //             // let chunk = compiler::compile(buf.clone());
    //             // vm::interpret(chunk);
    //         }
    //         Err(_) => break,
    //     }
    // }
}
fn run_file(file_path: &str) {
    // let mut vm = Vm::new();
    match fs::read_to_string(file_path) {
        // Ok(file) => _ = vm.interpret(file),
        Ok(source) => {
            // let chunk = compiler::compile(source);
            let chunk = compiler::compile(source);
            vm::interpret(chunk, stdout());
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
    println!("Bofink compiler stopped...");
}

#[cfg(test)]
mod tests {
    use crate::{compiler, vm};

    fn test_code(source: &str, expected_output: &str) {
        let chunk = compiler::compile(source.to_string());
        let mut buf = Vec::new();
        vm::interpret(chunk, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn hello_world() {
        let source = r#"
            print "Hello world!";
        "#;

        let expected = "Hello world!\n";

        test_code(source, expected);
    }

    #[test]
    fn concatenation() {
        let source = r#"
            int i = 1;
            str s = "word";
            print "Concatenating " + i + s;
        "#;
        let expected = "Concatenating 1word\n";

        test_code(source, expected)
    }

    #[test]
    fn adding_numbers() {
        let source = r#"
            int i = 2 + 3 + 4 + 5;
            print "Sum: " + i;
        "#;
        let expected = "Sum: 14\n";

        test_code(source, expected);
    }

    #[test]
    fn while_loop() {
        let source = r#"
            int i = 0;
            while i < 5 {
                print "hello";
                i = i + 1;
            }
        "#;
        let expected = "hello\nhello\nhello\nhello\nhello\n";

        test_code(source, expected);
    }

    #[test]
    fn while_loop_concat() {
        let source = r#"
            int i = 0;
            while i < 5 {
                print "hello" + i;
                i = i + 1;
            }
        "#;
        let expected = "hello0\nhello1\nhello2\nhello3\nhello4\n";

        test_code(source, expected);
    }

    #[test]
    fn function() {
        let source = r#"
            fun test(p1: int, p2: str) {
                print "first: " + p1;
                print "second: " + p2;
            }
            test(5, "a string");
            test(10, "text");
        "#;
        let expected = "first: 5\nsecond: a string\nfirst: 10\nsecond: text\n";

        test_code(source, expected);
    }

    #[test]
    fn function_with_local_params() {
        let source = r#"
            str s1 = "first";
            fun test(p1: str, p2: str) {
                print p1 + "!";
                print p2 + "!";
            }
            str s2 = "second";
            test(s1, s2);
        "#;
        let expected = "first!\nsecond!\n";

        test_code(source, expected);
    }

    #[test]
    fn multiple_function_with_local_params() {
        let source = r#"
            str s1 = "first";
            fun test(p1: str, p2: str) {
                print p1 + "!";
                print p2 + "!";
            }
            str s2 = "second";
            test(s1, s2);
            test(s1, s2);
        "#;
        let expected = "first!\nsecond!\nfirst!\nsecond!\n";

        test_code(source, expected);
    }

    #[test]
    fn if_statement() {
        let source = r#"
            if true {
                print "yes1";
            }
            if false {
                print "no";
            }
            if 1 == 1 {
                print "yes2";
            }
            if "hello" != "world" {
                print "yes3";
            }

            int i = 0;
            i = i + 1;
            if i == 1 {
                print "yes4";
            }
        "#;
        let expected = "yes1\nyes2\nyes3\nyes4\n";

        test_code(source, expected);
    }
    #[test]
    fn nested_while() {
        let source = r#"
            int i = 0;
            while i < 3 {
                int j = 0;
                while j < 3 {
                    print "i: " + i + ", j: " + j;
                    j = j + 1;
                }
                i = i + 1;
            }
        "#;
        let expected = "i: 0, j: 0\ni: 0, j: 1\ni: 0, j: 2\ni: 1, j: 0\ni: 1, j: 1\ni: 1, j: 2\ni: 2, j: 0\ni: 2, j: 1\ni: 2, j: 2\n";

        test_code(source, expected);
    }

    #[test]
    fn nested_while_with_if() {
        let source = r#"
            int i = 0;
            while i < 5 {
                int j = 0;
                while j < 5 {
                    if j == 3 {
                        print "j==3";
                    }
                    j = j + 1;
                }
                i = i + 1;
            }
        "#;
        let expected = "j==3\n".repeat(5);

        test_code(source, &expected);
    }

    #[test]
    fn for_loop() {
        let source = r#"
            for int i = 0; i < 3; i = i + 1; {
                print "i" + i;
            }
        "#;
        let expected = "i0\ni1\ni2\n";

        test_code(source, &expected);
    }

    #[test]
    fn nested_for_loop() {
        let source = r#"
            for int i = 0; i < 3; i = i + 1; {
                for int j = 0; j < 3; j = j + 1; {
                    print "i" + i + "j" + j;
                }
            }
        "#;
        let expected = "i0j0\ni0j1\ni0j2\ni1j0\ni1j1\ni1j2\ni2j0\ni2j1\ni2j2\n";

        test_code(source, &expected);
    }

    #[test]
    fn and_and_or() {
        let source = r#"
            if true and true {
                print "1";
            }
            if true and false {
                print "2";
            }
            if false and true {
                print "3";
            }
            if false and false {
                print "4";
            }

            if true or true {
                print "5";
            }
            if true or false {
                print "6";
            }
            if false or true {
                print "7";
            }
            if false or false {
                print "8";
            }
        "#;
        let expected = "1\n5\n6\n7\n";

        test_code(source, &expected);
    }
}
