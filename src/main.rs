use std::env;
use std::process;
use lox::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 2 {
        println!("Too many arguments");
        println!("Usage: lox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1]) {
            println!("File error: {e}");
            process::exit(1);
        }
    } else {
        let _ = run_prompt();
    }
}
