use std::{env, fs, io::stdout};

mod compiler;
mod enums;
mod opcode;
mod scanner;
mod vm;

fn main() {
    println!("Bofink compiler started...");
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => run_file(args.last().unwrap()),
        _ => panic!("Unacceptable usage"),
    }
    println!("Bofink compiler stopped...");
}

fn run_file(file_path: &str) {
    match fs::read_to_string(file_path) {
        Ok(source) => match compiler::compile(source) {
            Ok(chunk) => vm::start(chunk, &mut stdout()),
            Err(_) => {
                println!("Error compiling file {}", file_path);
                // println!("{}", source.lines().nth(e.))
                // println!("Compiler error: {}", e)
            }
        },
        Err(_) => panic!("Error reading file"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{compiler, vm};

    fn test_output(source: &str, expected_output: &str) {
        let mut buf = Vec::new();
        match compiler::compile(source.to_string()) {
            Ok(chunk) => vm::start(chunk, &mut buf),
            Err(e) => panic!("Compiler error: {}", e),
        }
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, expected_output);
    }

    // TODO: test error types
    fn test_error(source: &str) {
        let _result = compiler::compile(source.to_string());
    }

    #[test]
    fn assignment_without_declaration_should_fail() {
        test_error("some code");
    }

    #[test]
    fn hello_world() {
        let source = r#"
            print "Hello world!";
        "#;

        let expected = "Hello world!\n";

        test_output(source, expected);
    }

    #[test]
    fn concatenation() {
        let source = r#"
            let i = 1;
            let s = "word";
            let b = true;
            print s + i;
            print s + b;
            print i + s;
            print b + s;
        "#;
        let expected = "word1\nwordtrue\n1word\ntrueword\n";

        test_output(source, expected)
    }

    #[test]
    fn adding_numbers() {
        let source = r#"
            let i = 2 + 3 + 4 + 5;
            print "Sum: " + i;
        "#;
        let expected = "Sum: 14\n";

        test_output(source, expected);
    }

    #[test]
    fn while_loop() {
        let source = r#"
            mut i = 0;
            while i < 5 {
                print "hello";
                i = i + 1;
            }
        "#;
        let expected = "hello\nhello\nhello\nhello\nhello\n";

        test_output(source, expected);
    }

    #[test]
    fn while_loop_concat() {
        let source = r#"
            mut i = 0;
            while i < 5 {
                print "hello" + i;
                i = i + 1;
            }
        "#;
        let expected = "hello0\nhello1\nhello2\nhello3\nhello4\n";

        test_output(source, expected);
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

        test_output(source, expected);
    }

    #[test]
    fn function_with_local_params() {
        let source = r#"
            let s1 = "first";
            fun test(p1: str, p2: str) {
                print p1 + "!";
                print p2 + "!";
            }
            let s2 = "second";
            test(s1, s2);
        "#;
        let expected = "first!\nsecond!\n";

        test_output(source, expected);
    }

    #[test]
    fn multiple_function_with_local_params() {
        let source = r#"
            let s1 = "first";
            fun test(p1: str, p2: str) {
                print p1 + "!";
                print p2 + "!";
            }
            let s2 = "second";
            test(s1, s2);
            test(s1, s2);
        "#;
        let expected = "first!\nsecond!\nfirst!\nsecond!\n";

        test_output(source, expected);
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

            mut i = 0;
            i = i + 1;
            if i == 1 {
                print "yes4";
            }
        "#;
        let expected = "yes1\nyes2\nyes3\nyes4\n";

        test_output(source, expected);
    }
    #[test]
    fn nested_while() {
        let source = r#"
            mut i = 0;
            while i < 3 {
                mut j = 0;
                while j < 3 {
                    print "i: " + i + ", j: " + j;
                    j = j + 1;
                }
                i = i + 1;
            }
        "#;
        let expected = "i: 0, j: 0\ni: 0, j: 1\ni: 0, j: 2\ni: 1, j: 0\ni: 1, j: 1\ni: 1, j: 2\ni: 2, j: 0\ni: 2, j: 1\ni: 2, j: 2\n";

        test_output(source, expected);
    }

    #[test]
    fn nested_while_with_if() {
        let source = r#"
            mut i: int = 0;
            while i < 5 {
                mut j: int = 0;
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

        test_output(source, &expected);
    }

    #[test]
    fn for_loop() {
        let source = r#"
            for i in 0:3 {
                print "i" + i;
            }
        "#;
        let expected = "i0\ni1\ni2\n";

        test_output(source, &expected);
    }

    #[test]
    fn nested_for_loop() {
        let source = r#"
            for i in 0:3 {
                for j in 0:3 {
                    print "i" + i + "j" + j;
                }
            }
        "#;
        let expected = "i0j0\ni0j1\ni0j2\ni1j0\ni1j1\ni1j2\ni2j0\ni2j1\ni2j2\n";

        test_output(source, &expected);
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

        test_output(source, &expected);
    }

    #[test]
    fn compare_strings() {
        let source = r#"
        if "1" == "1" {
            print "1";
        }
        if "1" == "2" {
            print "error";
        }
        if "1" != "2" {
            print "2";
        }
        if "2" != "2" {
            print "error";
        }
        "#;
        let expected = "1\n2\n";
        test_output(source, expected);
    }

    #[test]
    fn comparing() {
        let source = r#"
            let i = 5;
            if i > 3 {
                print "3";
            }
            if i < 8 {
                print "8";
            }
            if i < 3 {
                print "not this";
            }
            if i > 8 {
                print "not this";
            }
            if i == 5 {
                print "5";
            }
            if i != 5 {
                print "not this";
            }
            if i != 7 {
                print "7";
            }
            if 5 <= 5 {
                print "5";
            }
            if 5 >= 5 {
                print "5";
            }
        "#;
        let expected = "3\n8\n5\n7\n5\n5\n";
        test_output(source, expected)
    }

    #[test]
    fn arithmetic() {
        let source = r#"
            let i = 5 - 3;
            if i != 2 {
                print "something went wrong";
            }

            let j = 5 + 5;
            if j != 10 {
                print "something went wrong";
            }

            let k = 10 / 2;
            if k != 5 {
                print "something went wrong";
            }

            let l = 5 * 5;
            if l != 25 {
                print "something went wrong";
            }

            let h = 7 % 3;
            if h != 1 {
                print "something went wrong";
            }
        "#;
        let expected = "";
        test_output(source, expected)
    }

    #[test]
    fn recursion() {
        let source = r#"
            fun fib(i: int) int {
                if i == 0 {
                    return 1;
                }
                if i == 1 {
                    return 1;
                }
                let r1 = fib(i - 1);
                let r2 = fib(i - 2);
                return r1 + r2; 
            }
            let res: int = fib(10);
            print "res: " + res;
        "#;
        let expected = "res: 89\n";
        test_output(source, expected);
    }
    #[test]
    fn fizzbuzz() {
        let source = r#"
            for i in 1:20 {
                if i % 3 == 0 and i % 5 == 0 {
                    print "fizzbuzz";
                }
                if i % 3 == 0 and i % 5 != 0 {
                    print "fizz";
                }
                if i % 3 != 0 and i % 5 == 0 {
                    print "buzz";
                }
                if i % 3 != 0 and i % 5 != 0 {
                    print "" + i;
                }
            }
        "#;
        let expected = "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz\n16\n17\nfizz\n19\n";
        test_output(source, expected);
    }

    #[test]
    fn nested_objects_get() {
        let source = r#"
            class Test {
                int i1;
            }
            class Foo {
                Test t1;
            }
            let bar = new Foo(new Test(2));
            print "i1:" + bar.t1.i1;
        "#;
        let expected = "i1:2\n";
        test_output(source, expected);
    }
}
