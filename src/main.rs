use lox::*;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let _ = run_prompt();
    } else {
        let mut command_line_args = vec![];
        command_line_args.extend(args.iter().skip(2).map(|arg| arg.as_str()));
        if let Err(e) = run_file(&args[1], &command_line_args) {
            println!("File error: {e}");
            process::exit(1);
        }
    }
}
