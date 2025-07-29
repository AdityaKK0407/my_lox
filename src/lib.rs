use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
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
    let mut env = Environment::new(None);
    run(&contents[..], &mut env, false);
    Ok(())
}

pub fn run_prompt() -> Result<(), String> {
    let mut statement = String::new();
    let mut env = Environment::new(None);
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut statement)
            .expect("Failed to read line");

        if statement.trim() == "exit" {
            break;
        }
        run(&statement[..], &mut env, true);
        statement.clear();
    }

    Ok(())
}

fn run(source_code: &str, env: &mut Rc<RefCell<Environment>>, is_repl: bool) {
    let tokenizer = lexer::Tokenizer::new(source_code.to_string());
    let (tokens, had_error) =  tokenizer.scan_tokens();

    if had_error {
        return;
    }

    let program = parser::parser::produce_ast(tokens);
    let parsed_program;
    match program {
        Ok(s) => parsed_program = s,
        Err(e) => {
            println!("{:#?}", e);
            handle_parser_error(e);
            return;
        }
    }
    if let Err(e) = interpreter::interpreter::evaluate_program(parsed_program, env, is_repl) {
        handle_runtime_error(e);
    }
}
