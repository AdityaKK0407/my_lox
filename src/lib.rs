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
    pub mod statement;
    pub mod parser;
}
mod global_scope;
mod values;

pub fn run_file(file_path: &str, command_line_args: &[&str]) -> Result<(), Box<dyn Error>> {
    if !file_path.ends_with(".lox") {
        return Err("Invalid file type, expected a .lox file".into());
    }
    let contents = fs::read_to_string(file_path)?;
    let mut env = Environment::new(None);
    run(&contents[..], &mut env, command_line_args, false);
    Ok(())
}

pub fn run_prompt() {
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
        run(&statement[..], &mut env, &vec![], true);
        statement.clear();
    }
}

fn run(
    source_code: &str,
    env: &mut Rc<RefCell<Environment>>,
    command_line_args: &[&str],
    is_repl: bool,
) {
    let serialized_code = serialize_source_code(source_code);

    let tokenizer = lexer::Tokenizer::new(source_code);
    let (tokens, had_error) = tokenizer.scan_tokens(&serialized_code);

    if had_error {
        return;
    }

    let mut program = parser::parser::Parser::new(tokens, is_repl);
    let parsed_program = match program.produce_ast() {
        Ok(s) => s,
        Err(e) => {
            handle_parser_error(e, &serialized_code);
            return;
        }
    };

    if let Err(e) =
        interpreter::interpreter::evaluate_program(&parsed_program, env, command_line_args, is_repl)
    {
        handle_runtime_error(e, &serialized_code);
    }
}

fn serialize_source_code(code: &str) -> Vec<&str> {
    let mut result = vec![];

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push("...");
        } else {
            result.push(trimmed);
        }
    }

    result
}
