use std::cell::RefCell;
use std::error::Error;
use std::fs;
// use std::io;
// use std::io::Write;
use std::rc::Rc;

use crate::environment::*;
use crate::handle_errors::*;

mod ast;
mod environment;
mod handle_errors;
mod interpreter {
    pub mod expression;
    pub mod interpreter;
    pub mod statement;
}
mod lexer;
mod parser {
    pub mod expression;
    pub mod parser;
    pub mod statement;
}
mod global_scope;
mod values;

pub fn run_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let env = Environment::new(None);
    run(&contents[..], Rc::clone(&env), false);
    Ok(())
}

// pub fn run_prompt() -> Result<(), String> {
//     let env = Environment::new(None);
//     loop {
//         let mut statement = String::new();
//         print!("> ");
//         io::stdout().flush().unwrap();

//         io::stdin()
//             .read_line(&mut statement)
//             .expect("Failed to read line");

//         if statement.trim() == "exit" {
//             break;
//         }
//         run(statement, Rc::clone(&env), true);
//     }

//     Ok(())
// }

fn run<'a>(source_code: &'a str, env: Rc<RefCell<Environment<'a>>>, is_repl: bool) {
    let tokenizer = lexer::Tokenizer::new(source_code);
    let (tokens, had_error) = tokenizer.scan_tokens();

    if had_error {
        return;
    }

    let mut parser = parser::parser::Parser::new(tokens);

    let program = parser.produce_ast();
    let parsed_program;
    match program {
        Ok(s) => parsed_program = s,
        Err(e) => {
            println!("{:#?}", e);
            handle_parser_error(e);
            return;
        }
    }
    if let Err(e) = interpreter::interpreter::evaluate_program(parsed_program, &env, is_repl) {
        handle_runtime_error(e);
    }
}
