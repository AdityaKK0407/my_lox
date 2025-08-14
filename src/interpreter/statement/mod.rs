use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::io;
use std::io::Write;

use crate::ast::*;
use crate::environment::*;
use crate::handle_errors::*;
use crate::interpreter::expression::*;
use crate::interpreter::interpreter::*;
use crate::values::*;

pub fn var_declaration(
    declaration: &VarDeclaration,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalResult, RuntimeError> {
    let value = evaluate_expr(&declaration.value, env)?;
    if let Err(err) = declare_var(
        env,
        &declaration.identifier[..],
        value.clone(),
        declaration.constant,
    ) {
        if err == EnvironmentError::ReDeclareVar {
            return Err(RuntimeError::EnvironmentError(
                format!(
                    "{} is already declared. Cannot redeclare variable with same name",
                    declaration.identifier
                ),
                declaration.line,
            ));
        }
    }
    Ok(make_none())
}

pub fn print_stmt(
    value: &Option<Vec<Expr>>,
    env: &Rc<RefCell<Environment>>,
    new_line: bool,
) -> Result<EvalResult, RuntimeError> {
    if let Some(expr) = value {
        for expr in expr {
            let runtime_val = evaluate_expr(expr, env)?;
            print_runtime_val(runtime_val);
        }
    }
    if new_line {
        println!();
    }
    io::stdout().flush().unwrap();
    Ok(make_none())
}

pub fn print_runtime_val(runtime_val: RuntimeVal) {
    match runtime_val {
        RuntimeVal::Number(num) => print!("{}", num),
        RuntimeVal::Bool(bit) => print!("{}", bit),
        RuntimeVal::Nil => print!("nil"),
        RuntimeVal::String(s) => print!("{}", s),
        RuntimeVal::Object(obj) => print_obj(obj),
        RuntimeVal::Array(arr) => print_arr(arr),
        RuntimeVal::Function { name, .. } => print!("Function: '{}'", name),
        RuntimeVal::NativeFunction(_, name) => print!("Native Function: '{}'", name),
        RuntimeVal::Method { name, .. } => print!("Method '{}'", name),
        RuntimeVal::Class { name, .. } => print!("Class: '{}'", name),
        RuntimeVal::Instance { class_name, .. } => print!("Class Instance: '{}'", class_name),
    }
}

fn print_obj(obj: HashMap<String, RuntimeVal>) {
    println!("{{");
    for (key, value) in obj.iter() {
        print!("    \"{}\": ", key);
        print_runtime_val(value.clone());
        println!(",");
    }
    println!("}}");
}

fn print_arr(arr: Vec<RuntimeVal>) {
    print!("[");
    for val in arr {
        print_runtime_val(val);
        print!(", ");
    }
    println!("]");
}

pub fn if_else_stmt(
    collection: &[(Expr, Vec<Stmt>, usize)],
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalResult, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    let mut is_if_stmt = true;
    for (expr, statements, line) in collection {
        let condition = evaluate_expr(expr, &local_env)?;
        if let RuntimeVal::Bool(bit) = condition {
            is_if_stmt = false;
            if !bit {
                continue;
            } else {
                for statement in statements {
                    match evaluate(&statement, &local_env)? {
                        EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                        EvalResult::Break => return Ok(EvalResult::Break),
                        EvalResult::Continue => return Ok(EvalResult::Continue),
                        _ => continue,
                    }
                }
                break;
            }
        }
        let str: &str = if is_if_stmt { "if" } else { "else-if" };
        return Err(RuntimeError::TypeMismatch(
            format!("Expressions of {} statements must be of type bool", str),
            *line,
        ));
    }
    Ok(make_none())
}

pub fn for_stmt(
    stmt: &Stmt,
    expr1: &Expr,
    expr2: &Expr,
    statements: &[Stmt],
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<EvalResult, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    let _ = evaluate(&stmt, &local_env)?;

    loop {
        if let RuntimeVal::Bool(bit) = evaluate_expr(expr1, &local_env)? {
            if !bit {
                break;
            }
            for statement in statements {
                match evaluate(&statement, &local_env)? {
                    EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                    EvalResult::Break => return Ok(make_none()),
                    EvalResult::Continue => break,
                    _ => continue,
                }
            }
            let _ = evaluate(&Stmt::Expression(expr2.clone()), &local_env)?;
        } else {
            return Err(RuntimeError::TypeMismatch(
                "Only bool type allowed in for loop condition statement".into(),
                line,
            ));
        }
    }

    Ok(make_none())
}

pub fn while_stmt(
    expr: &Expr,
    statements: &[Stmt],
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<EvalResult, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    loop {
        if let RuntimeVal::Bool(bit) = evaluate_expr(expr, &local_env)? {
            if !bit {
                break;
            }
            for statement in statements {
                match evaluate(&statement, &local_env)? {
                    EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                    EvalResult::Break => return Ok(make_none()),
                    EvalResult::Continue => break,
                    _ => continue,
                }
            }
        } else {
            return Err(RuntimeError::TypeMismatch(
                "Only bool type allowed in for loop condition statement".into(),
                line,
            ));
        }
    }

    Ok(make_none())
}

pub fn block_stmt(
    stmts: Vec<Stmt>,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalResult, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    for stmt in stmts {
        match evaluate(&stmt, &local_env)? {
            EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
            EvalResult::Break => return Ok(EvalResult::Break),
            EvalResult::Continue => return Ok(EvalResult::Continue),
            _ => continue,
        }
    }
    Ok(make_none())
}
