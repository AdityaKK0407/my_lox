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
    let serialized_code = serialize_source_code(source_code);

    let tokenizer = lexer::Tokenizer::new(source_code);
    let (tokens, had_error) = tokenizer.scan_tokens(&serialized_code);

    if had_error {
        return;
    }

    let mut program = parser::parser::Parser::new(tokens);
    let parsed_program;
    match program.produce_ast() {
        Ok(s) => parsed_program = s,
        Err(e) => {
            handle_parser_error(e, &serialized_code);
            return;
        }
    }
    if let Err(e) = interpreter::interpreter::evaluate_program(&parsed_program, env, is_repl) {
        handle_runtime_error(e);
    }
}

fn serialize_source_code(code: &str) -> Vec<String> {
    let mut result = vec![];

    let mut temp = String::new();
    for c in code.chars() {
        if c == '\n' {
            if temp.is_empty() {
                temp = String::from("...");
            }
            result.push(temp);
            temp = String::new();
        } else if c != '\r' {
            temp.push(c);
        }
    }
    if temp.is_empty() {
        temp = String::from("...");
    }
    result.push(temp);
    result
}
