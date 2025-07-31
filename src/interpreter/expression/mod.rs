use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::handle_errors::RuntimeError;
use crate::interpreter::interpreter::*;
use crate::lexer::*;
use crate::values::*;

pub fn evaluate_expr(
    expr: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match expr {
        Expr::NumericLiteral(num) => Ok(make_number(*num)),
        Expr::Null => Ok(make_nil()),
        Expr::BoolLiteral(bit) => Ok(make_bool(*bit)),
        Expr::StringLiteral(str) => Ok(make_string(&str[..])),
        Expr::Identifier(symbol) => evaluate_identifier(&symbol[..], env),
        Expr::This => evaluate_identifier("this", env),
        Expr::Member {
            object,
            property,
            computed,
        } => evaluate_member_expr(object, property, *computed, env),
        Expr::Call { args, caller } => evaluate_function_call(args, caller, env),
        Expr::Unary { operator, right } => {
            evaluate_unary_expr(operator, right, env)
        }
        Expr::BinaryExpr {
            left,
            operator,
            right,
        } => evaluate_binary_expr(left, operator, right, env),
        Expr::ComparisonLiteral {
            left,
            operator,
            right,
        } => evaluate_compare_expr(left, operator, right, env),
        Expr::ObjectLiteral { properties } => evaluate_object_expr(properties, env),
        Expr::AssignmentExpr { assignee, value } => {
            evaluate_assignment(assignee, value, env)
        }
    }
}

fn evaluate_numeric_binary_expr(lhs: f64, rhs: f64, operator: &str) -> RuntimeVal {
    make_number(match operator {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        _ => lhs % rhs,
    })
}

fn evaluate_binary_expr(
    left: &Expr,
    operator: &Token,
    right: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let left_hand_side = evaluate_expr(left, env)?;
    let right_hand_side = evaluate_expr(right, env)?;
    if let RuntimeVal::Number(lhs) = left_hand_side {
        if let RuntimeVal::Number(rhs) = right_hand_side {
            return Ok(evaluate_numeric_binary_expr(lhs, rhs, &operator.lexeme[..]));
        }
    }
    Err(RuntimeError::MisMatchTypes)
}

fn evaluate_assignment(
    assignee: &Expr,
    value: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match assignee {
        Expr::Identifier(ident) => {
            let value = evaluate_expr(value, env)?;
            return assign_var(env, &ident[..], value);
        }
        Expr::Member {
            object,
            property,
            computed,
        } => {
            let _ = equate_member_expr(object, property, *computed, value, env);
            return evaluate_expr(value, env);
        }
        _ => panic!(),
    }
}

fn evaluate_identifier(
    ident: &str,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    lookup_var(env, ident)
}

fn evaluate_object_expr(
    obj: &[Property],
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut map = HashMap::new();

    for prop in obj {
        let runtime_val;
        if let Some(expr) = &prop.value {
            runtime_val = evaluate_expr(&expr, env)?;
        } else {
            runtime_val = lookup_var(env, &prop.key[..])?;
        }
        map.insert(prop.key.clone(), runtime_val);
    }
    Ok(make_obj(&map))
}

fn evaluate_compare_expr(
    left: &Expr,
    operator: &Token,
    right: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let left_hand_side = evaluate_expr(left, env)?;
    let right_hand_side = evaluate_expr(right, env)?;

    if operator.token_type == TokenType::AND || operator.token_type == TokenType::OR {
        return evaluate_logical_expr(left_hand_side, right_hand_side, &operator.lexeme[..]);
    } else if operator.token_type == TokenType::EQUALEQUAL
        || operator.token_type == TokenType::BANGEQUAL
    {
        return evaluate_equality_expr(left_hand_side, right_hand_side, &operator.lexeme[..]);
    } else {
        return evaluate_comparison_expr(left_hand_side, right_hand_side, &operator.lexeme[..]);
    }
}

fn evaluate_logical_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
) -> Result<RuntimeVal, RuntimeError> {
    if let RuntimeVal::Bool(lhs) = left {
        if let RuntimeVal::Bool(rhs) = right {
            return match operator {
                "and" => Ok(make_bool(lhs && rhs)),
                _ => Ok(make_bool(lhs || rhs)),
            };
        }
    }
    Err(RuntimeError::MisMatchTypes)
}

fn evaluate_equality_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
) -> Result<RuntimeVal, RuntimeError> {
    if let RuntimeVal::Number(num1) = left {
        if let RuntimeVal::Number(num2) = right {
            return Ok(make_bool(match operator {
                "==" => num1 == num2,
                _ => num1 != num2,
            }));
        }
    }

    if let RuntimeVal::Bool(bit1) = left {
        if let RuntimeVal::Bool(bit2) = right {
            return Ok(make_bool(match operator {
                "==" => bit1 == bit2,
                _ => bit1 != bit2,
            }));
        }
    }

    if let RuntimeVal::String(str1) = left {
        if let RuntimeVal::String(str2) = right {
            return Ok(make_bool(match operator {
                "==" => str1 == str2,
                _ => str1 != str2,
            }));
        }
    }

    Err(RuntimeError::MisMatchTypes)
}

fn evaluate_comparison_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
) -> Result<RuntimeVal, RuntimeError> {
    if let RuntimeVal::Number(num1) = left {
        if let RuntimeVal::Number(num2) = right {
            return Ok(make_bool(match operator {
                ">" => num1 > num2,
                ">=" => num1 >= num2,
                "<" => num1 < num2,
                _ => num1 <= num2,
            }));
        }
    }

    if let RuntimeVal::Bool(bit1) = left {
        if let RuntimeVal::Bool(bit2) = right {
            return Ok(make_bool(match operator {
                ">" => bit1 > bit2,
                ">=" => bit1 >= bit2,
                "<" => bit1 < bit2,
                _ => bit1 <= bit2,
            }));
        }
    }

    if let RuntimeVal::String(str1) = left {
        if let RuntimeVal::String(str2) = right {
            return Ok(make_bool(match operator {
                ">" => str1 > str2,
                ">=" => str1 >= str2,
                "<" => str1 < str2,
                _ => str1 <= str2,
            }));
        }
    }

    Err(RuntimeError::MisMatchTypes)
}

fn evaluate_unary_expr(
    operator: &Token,
    right: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let value = evaluate_expr(right, env)?;

    if operator.token_type == TokenType::BANG {
        if let RuntimeVal::Bool(bit) = value {
            return Ok(make_bool(!bit));
        }
        panic!("NOT operator not compactible with non-bool types");
    } else {
        if let RuntimeVal::Number(num) = value {
            return Ok(make_number(-num));
        }
        panic!("Negation only works for numbers");
    }
}

fn evaluate_function_call(
    args: &[Expr],
    caller: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let call = evaluate_expr(caller, env)?;

    function_call(args, call, env)
}

fn function_call(
    args: &[Expr],
    call: RuntimeVal,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match call {
        RuntimeVal::Class { name, methods, .. } => {
            let instance_env = Environment::new(None);
            let constructor = methods.get(name.as_str());
            let instance = make_instance(&name[..], instance_env);
            match constructor {
                Some(func) => {
                    if let RuntimeVal::Function {
                        params,
                        body,
                        closure,
                        ..
                    } = func
                    {
                        let local_env = Environment::new(Some(Rc::clone(&closure)));
                        declare_var(&local_env, "this", instance.clone(), true)?;
                        if args.len() < params.len() {
                            return Err(RuntimeError::LessFuncArguments);
                            // panic!("Invalid number of arguments to function {name}");
                        }
                        if args.len() > params.len() {
                            return Err(RuntimeError::MoreFuncArguments);
                            // panic!("Invalid number of arguments to function {name}");
                        }
                        for i in 0..args.len() {
                            let value = evaluate_expr(&args[i], &local_env)?;
                            declare_var(&local_env, &params[i][..], value, false)?;
                        }

                        for stmt in body {
                            match evaluate(&stmt, &local_env)? {
                                EvalResult::Return(_) => break,
                                _ => continue,
                            }
                        }
                    }
                }
                None => {}
            }
            return Ok(instance);
        }
        RuntimeVal::Method { func, instance } => {
            if let RuntimeVal::Function {
                params,
                body,
                closure,
                ..
            } = *func
            {
                let local_env = Environment::new(Some(Rc::clone(&closure)));
                declare_var(&local_env, "this", *instance, true)?;
                if args.len() < params.len() {
                    return Err(RuntimeError::LessFuncArguments);
                    // panic!("Invalid number of arguments to function {name}");
                }
                if args.len() > params.len() {
                    return Err(RuntimeError::MoreFuncArguments);
                    // panic!("Invalid number of arguments to function {name}");
                }
                for i in 0..args.len() {
                    let value = evaluate_expr(&args[i], env)?;
                    declare_var(&local_env, &params[i][..], value, false)?;
                }

                for stmt in body {
                    match evaluate(&stmt, &local_env)? {
                        EvalResult::Return(val) => return Ok(val),
                        _ => continue,
                    }
                }
            }
        }
        RuntimeVal::Function {
            params,
            body,
            closure,
            ..
        } => {
            let local_env = Environment::new(Some(Rc::clone(&closure)));
            if args.len() < params.len() {
                return Err(RuntimeError::LessFuncArguments);
                // panic!("Invalid number of arguments to function {name}");
            }
            if args.len() > params.len() {
                return Err(RuntimeError::MoreFuncArguments);
                // panic!("Invalid number of arguments to function {name}");
            }
            for i in 0..args.len() {
                let value = evaluate_expr(&args[i], env)?;
                declare_var(&local_env, &params[i][..], value, false)?;
            }

            for stmt in body {
                match evaluate(&stmt, &local_env)? {
                    EvalResult::Return(val) => return Ok(val),
                    _ => continue,
                }
            }
        }
        RuntimeVal::NativeFunction(func) => {
            let mut values = vec![];
            for arg in args {
                values.push(evaluate_expr(&arg, env)?);
            }
            return func(values);
        }
        _ => panic!("Invalid for function call"),
    }

    Ok(make_nil())
}

fn evaluate_member_expr(
    object: &Expr,
    property: &Expr,
    computed: bool,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut obj = evaluate_expr(object, env)?;

    if computed {
        let key = evaluate_expr(property, env)?;
        match (obj, key) {
            (RuntimeVal::Object(map), RuntimeVal::String(str)) => {
                let value = map.get(str.as_str());
                match value {
                    Some(val) => return Ok(val.clone()),
                    None => return Ok(make_nil()),
                }
            }
            _ => return Err(RuntimeError::MisMatchTypes),
        }
    } else {
        let lexeme;
        if let Expr::Identifier(name) = property {
            lexeme = name;
        } else {
            return Err(RuntimeError::MisMatchTypes);
        }
        let mut method_exists = None;
        loop {
            match obj {
                RuntimeVal::Object(map) => {
                    let res = map.get(lexeme.clone().as_str());
                    match res {
                        Some(value) => return Ok(value.clone()),
                        None => return Err(RuntimeError::ObjectField),
                    }
                }
                RuntimeVal::Class {
                    static_fields,
                    methods,
                    superclass,
                    ..
                } => {
                    let method = methods.get(lexeme);
                    if let Some(method) = method {
                        if let Some(val) = method_exists {
                            return Ok(make_method(method.clone(), val));
                        }
                        return Ok(method.clone());
                    }
                    let static_field = static_fields.get(lexeme);
                    if let Some(static_field) = static_field {
                        return Ok(static_field.clone());
                    }
                    match superclass {
                        Some(parent) => {
                            obj = lookup_var(env, &parent[..])?;
                        }
                        None => {
                            panic!("This doesn't exist in class or parent classes");
                        }
                    }
                }
                RuntimeVal::Instance {
                    class_name,
                    instance_env,
                } => match lookup_var(&instance_env, &lexeme[..]) {
                    Ok(value) => return Ok(value),
                    Err(_) => match lookup_var(&env, &class_name[..]) {
                        Ok(class) => {
                            method_exists =
                                Some(make_instance(&class_name[..], Rc::clone(&instance_env)));
                            obj = class;
                            continue;
                        }
                        Err(_) => return Err(RuntimeError::UnReachableError),
                    },
                },
                _ => return Err(RuntimeError::InvalidMemberExpr),
            }
        }
    }
}

fn equate_member_expr(
    object: &Expr,
    property: &Expr,
    computed: bool,
    value: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    // computed -> [] -> array, vector, etc
    // not -> . -> class, objects

    let result = evaluate_expr(value, env)?;
    let obj = evaluate_expr(object, env)?;

    if computed {
        let key = evaluate_expr(property, env)?;
        match (obj, key) {
            (RuntimeVal::Object(mut map), RuntimeVal::String(str)) => {
                map.insert(str, result.clone());
            }
            _ => panic!(),
        }
    } else {
        let lexeme;
        if let Expr::Identifier(name) = property {
            lexeme = name;
        } else {
            return Err(RuntimeError::MisMatchTypes);
        }
        match obj {
            RuntimeVal::Object(mut map) => {
                map.insert(lexeme.clone(), result.clone());
            }
            RuntimeVal::Class {
                mut static_fields,
                methods,
                ..
            } => {
                let method = methods.get(lexeme);
                if let Some(_) = method {
                    return Err(RuntimeError::MisMatchTypes);
                }
                static_fields.insert(lexeme.clone(), result.clone());
            }
            RuntimeVal::Instance { instance_env, .. } => {
                if let Err(_) = declare_var(&instance_env, &lexeme[..], result.clone(), false) {
                    assign_var(&instance_env, &lexeme[..], result.clone())?;
                }
            }
            _ => panic!(),
        }
    }
    Ok(result)
}
