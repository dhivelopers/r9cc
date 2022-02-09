use std::env;

fn main() {
    let ret_value = env::args().nth(1).unwrap();
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("\tmov rax, {}", ret_value);
    println!("\tret");
}
