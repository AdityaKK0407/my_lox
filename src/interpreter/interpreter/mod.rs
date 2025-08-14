use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::handle_errors::RuntimeError;
use crate::interpreter::expression::*;
use crate::interpreter::statement::*;
use crate::values::*;

pub fn evaluate_program(
    program: &[Stmt],
    env: &Rc<RefCell<Environment>>,
    command_line_args: &[&str],
    is_repl: bool,
) -> Result<(), RuntimeError> {
    let _ = evaluate_first_pass(program, env, is_repl)?;
    if is_repl {
        for statement in program {
            if let EvalResult::Value(val) = evaluate(&statement, env)? {
                print_runtime_val(val);
                println!();
            }
        }
    } else {
        let mut args = vec![];
        args.extend(
            command_line_args
                .iter()
                .map(|s| Expr::StringLiteral(s.to_string(), 0)),
        );
        let main_stmt = Stmt::Expression(Expr::Call {
            args,
            caller: Box::new(Expr::Identifier(String::from("main"), 0)),
            line: 0,
        }); // Calling main function happens outside the code, thus denoted by line 0. NOT A MISTAKE
        evaluate(&main_stmt, env)?;
    }
    Ok(())
}

fn evaluate_first_pass(
    program: &[Stmt],
    env: &Rc<RefCell<Environment>>,
    is_repl: bool,
) -> Result<(), RuntimeError> {
    for statement in program {
        match statement {
            Stmt::Function(function) => {
                let func = make_function(
                    &function.name[..],
                    &function.parameters,
                    &function.body,
                    env,
                );
                if let Err(_) = declare_var(env, &function.name[..], func, true) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "{} is already declared. Cannot redeclare variable with same name",
                            function.name
                        ),
                        function.line,
                    ));
                }
            }
            Stmt::Class(class) => {
                let mut fields = HashMap::new();
                for var in &class.static_fields {
                    let _ = var_declaration(var, env)?;
                    let res = evaluate_expr(&var.value, env)?;
                    fields.insert(var.identifier.clone(), res);
                }
                let mut methods = HashMap::new();
                for (name, func) in &class.methods {
                    let res = make_function(&func.name[..], &func.parameters, &func.body, env);
                    methods.insert(name.clone(), res);
                }
                let class_val =
                    make_class(&class.name[..], fields, methods, class.superclass.clone());
                if let Err(_) = declare_var(env, &class.name[..], class_val, true) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "{} is already declared. Cannot redeclare variable with same name",
                            class.name
                        ),
                        class.line,
                    ));
                }
            }
            _ => {
                if !is_repl {
                    return Err(RuntimeError::InternalError);
                }
            }
        }
    }
    Ok(())
}

pub fn evaluate(
    ast_node: &Stmt,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalResult, RuntimeError> {
    match ast_node {
        Stmt::Expression(expr) => Ok(EvalResult::Value(evaluate_expr(expr, env)?)),
        Stmt::VarDeclaration(declaration) => var_declaration(declaration, env),
        Stmt::Print(value, new_line) => print_stmt(value, env, *new_line),
        Stmt::IfElse(if_collection) => if_else_stmt(if_collection, env),
        Stmt::While(expr, stmt, line) => while_stmt(expr, stmt, env, *line),
        Stmt::For((var_stmt, expr1, expr2), statement, line) => {
            for_stmt(var_stmt, expr1, expr2, statement, env, *line)
        }
        Stmt::Block(stmts) => block_stmt(stmts.clone(), env),
        Stmt::Return(expr) => Ok(make_return(evaluate_expr(expr, env)?)),
        Stmt::Break => Ok(make_break()),
        Stmt::Continue => Ok(make_continue()),
        Stmt::Function(FunctionDeclaration {
            name,
            parameters,
            body,
            line,
        }) => {
            let function = make_function(name, parameters, body, env);
            if let Err(_) = declare_var(env, &name[..], function, true) {
                return Err(RuntimeError::EnvironmentError(
                    format!(
                        "{} is already declared. Cannot redeclare variable with same name",
                        name
                    ),
                    *line,
                ));
            }
            Ok(make_none())
        }
        Stmt::Class(ClassDeclaration {
            name,
            static_fields,
            methods,
            superclass,
            line,
        }) => {
            let mut fields = HashMap::new();
            for var in static_fields {
                let _ = var_declaration(var, env)?;
                let value = evaluate_expr(&var.value, env)?;
                fields.insert(var.identifier.clone(), value);
            }
            let mut method = HashMap::new();
            for (name, func) in methods {
                let res = make_function(&func.name[..], &func.parameters, &func.body, env);
                method.insert(name.clone(), res);
            }
            let class_val = make_class(&name[..], fields, method, superclass.clone());
            if let Err(_) = declare_var(env, &name[..], class_val, true) {
                return Err(RuntimeError::EnvironmentError(
                    format!(
                        "{} is already declared. Cannot redeclare variable with same name",
                        name
                    ),
                    *line,
                ));
            }
            Ok(make_none())
        }
    }
}
