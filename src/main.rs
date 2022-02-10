use std::env;
use std::process;

fn main() {
    let ret_value = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: ./r9cc <number>");
        process::exit(1);
    });
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("\tmov rax, {ret_value}");
    println!("\tret");
}
