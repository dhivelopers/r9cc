use std::env;
use std::process;

use r9cc::codegen::Codegen as r9cc;

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage  : ./r9cc \"<code>\"");
        eprintln!("example: ./r9cc \"4+3+10-9\"");
        process::exit(1);
    });
    let out = r9cc::compile(&arg);
    match out {
        Ok(assemblys) => {
            println!("{}", assemblys.join("\n"));
        }
        Err(err) => {
            eprintln!("Error: (input) {}", arg);
            eprintln!("{:#?}", err);
        }
    }
}
