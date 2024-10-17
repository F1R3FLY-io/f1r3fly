use std::{fs::File, io::Read};

use rholang::rust::interpreter::compiler::normalizer::parser::parse_rholang_code_to_proc;

#[test]
fn parse_rholang_code_to_proc_function_should_successfully_parse_ground_literals() {
    let mut file = File::open("examples/stdout.rho").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // println!("{}", contents);

    let res = parse_rholang_code_to_proc(r#"x!(@"name"!("Joe") | @"age"!(40))"#);

    println!("{:?}", res);
    assert!(res.is_ok())
}
