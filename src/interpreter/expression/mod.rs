use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::handle_errors::EnvironmentError;
use crate::handle_errors::RuntimeError;
use crate::interpreter::interpreter::*;
use crate::lexer::*;
use crate::values::*;

pub fn evaluate_expr(
    expr: &Expr,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match expr {
        Expr::NumericLiteral(num, _) => Ok(make_number(*num)),
        Expr::Null(_) => Ok(make_nil()),
        Expr::BoolLiteral(bit, _) => Ok(make_bool(*bit)),
        Expr::StringLiteral(str, _) => Ok(make_string(&str[..])),
        Expr::Identifier(symbol, line) => evaluate_identifier(&symbol[..], env, *line),
        Expr::This(line) => evaluate_identifier("this", env, *line),
        Expr::Super(class_name, line) => evaluate_super_expr(class_name, env, *line),
        Expr::Array(array, _) => evaluate_array_expr(array, env),
        Expr::Member {
            object,
            property,
            computed,
            line,
        } => evaluate_member_expr(object, property, *computed, env, *line),
        Expr::Call { args, caller, line } => evaluate_function_call(args, caller, env, *line),
        Expr::Unary {
            operator,
            right,
            line,
        } => evaluate_unary_expr(operator, right, env, *line),
        Expr::BinaryExpr {
            left,
            operator,
            right,
            line,
        } => evaluate_binary_expr(left, operator, right, env, *line),
        Expr::ComparisonLiteral {
            left,
            operator,
            right,
            line,
        } => evaluate_compare_expr(left, operator, right, env, *line),
        Expr::ObjectLiteral { properties } => evaluate_object_expr(properties, env),
        Expr::AssignmentExpr {
            assignee,
            value,
            line,
        } => evaluate_assignment(assignee, value, env, *line),
    }
}

fn evaluate_unary_expr(
    operator: &Token,
    right: &Expr,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let value = evaluate_expr(right, env)?;
    if operator.token_type == TokenType::BANG {
        if let RuntimeVal::Bool(bit) = value {
            return Ok(make_bool(!bit));
        }
        Err(RuntimeError::TypeMismatch(
            "'!' NOT operator is only valid for bools".to_string(),
            line,
        ))
    } else {
        if let RuntimeVal::Number(num) = value {
            return Ok(make_number(-num));
        }
        Err(RuntimeError::TypeMismatch(
            "'-' negation operator is only valid for numbers".to_string(),
            line,
        ))
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
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let left_hand_side = evaluate_expr(left, env)?;
    let right_hand_side = evaluate_expr(right, env)?;
    if let RuntimeVal::Number(lhs) = left_hand_side {
        if let RuntimeVal::Number(rhs) = right_hand_side {
            return Ok(evaluate_numeric_binary_expr(lhs, rhs, &operator.lexeme[..]));
        }
    }
    Err(RuntimeError::TypeMismatch(
        format!(
            "{} operation is not valid for two non-numbers",
            operator.lexeme
        ),
        line,
    ))
}

fn evaluate_identifier(
    ident: &str,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    match lookup_var(env, ident) {
        Ok(val) => Ok(val),
        Err(_) => Err(RuntimeError::EnvironmentError(
            format!("'{}' is not declared.", ident),
            line,
        )),
    }
}

fn evaluate_super_expr(
    class_name: &str,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    if let Ok(class) = lookup_var(env, class_name) {
        if let RuntimeVal::Class {
            name, superclass, ..
        } = class
        {
            if let Some(parent_class) = superclass {
                return match lookup_var(env, &parent_class) {
                    Ok(val) => Ok(val),
                    Err(_) => {
                        Err(RuntimeError::EnvironmentError(
                            format!(
                                "Cannot use 'super' in '{}' class as parent class '{}' is not declared",
                                name, parent_class
                            ),
                            line,
                        ))
                    }
                }
            }
        }
    }
    Err(RuntimeError::InternalError)
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
            runtime_val = match lookup_var(env, &prop.key[..]) {
                Ok(val) => val,
                Err(_) => {
                    return Err(RuntimeError::EnvironmentError(
                        format!("{} is not declared yet.", prop.key),
                        prop.line,
                    ));
                }
            }
        }
        map.insert(prop.key.clone(), runtime_val);
    }
    Ok(make_obj(&map))
}

fn evaluate_array_expr(
    array: &[Expr],
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut val = vec![];

    for arr in array {
        val.push(evaluate_expr(arr, env)?);
    }

    Ok(make_arr(&val))
}

fn evaluate_compare_expr(
    left: &Expr,
    operator: &Token,
    right: &Expr,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let left_hand_side = evaluate_expr(left, env)?;
    let right_hand_side = evaluate_expr(right, env)?;

    if operator.token_type == TokenType::AND || operator.token_type == TokenType::OR {
        evaluate_logical_expr(left_hand_side, right_hand_side, &operator.lexeme[..], line)
    } else if operator.token_type == TokenType::EQUALEQUAL
        || operator.token_type == TokenType::BANGEQUAL
    {
        evaluate_equality_expr(left_hand_side, right_hand_side, &operator.lexeme[..], line)
    } else {
        evaluate_comparison_expr(
            left_hand_side,
            right_hand_side,
            &operator.lexeme[..],
            line,
        )
    }
}

fn evaluate_logical_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    if let RuntimeVal::Bool(lhs) = left {
        if let RuntimeVal::Bool(rhs) = right {
            return match operator {
                "and" => Ok(make_bool(lhs && rhs)),
                _ => Ok(make_bool(lhs || rhs)),
            };
        }
    }
    Err(RuntimeError::TypeMismatch(
        format!("{} logical operation is only valid for bools", operator),
        line,
    ))
}

fn evaluate_equality_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
    line: usize,
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

    Err(RuntimeError::TypeMismatch(
        format!(
            "{} equality operation is only valid for numbers, bools and strings",
            operator
        ),
        line,
    ))
}

fn evaluate_comparison_expr(
    left: RuntimeVal,
    right: RuntimeVal,
    operator: &str,
    line: usize,
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

    Err(RuntimeError::TypeMismatch(
        format!(
            "{} comparison operation is only valid for numbers, bools and strings",
            operator
        ),
        line,
    ))
}

fn evaluate_assignment(
    assignee: &Expr,
    value: &Expr,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    match assignee {
        Expr::Identifier(ident, line) => {
            let value = evaluate_expr(value, env)?;
            match assign_var(env, &ident[..], value) {
                Ok(val) => {
                    Ok(val)
                }
                Err(err) => match err {
                    EnvironmentError::ConstReassign => {
                        Err(RuntimeError::EnvironmentError(
                            format!(
                                "{} is a constant. Constant values cannot be reassigned",
                                ident
                            ),
                            *line,
                        ))
                    }
                    EnvironmentError::VarNotDeclared => {
                        Err(RuntimeError::EnvironmentError(
                            format!("{} has not been declared yet.", ident),
                            *line,
                        ))
                    }
                    EnvironmentError::ReDeclareVar => {
                        Err(RuntimeError::InternalError)
                    }
                },
            }
        }
        Expr::Member {
            object,
            property,
            computed,
            line,
        } => {
            let _ = equate_member_expr(object, property, *computed, value, env, *line);
            evaluate_expr(value, env)
        }
        _ => Err(RuntimeError::TypeMismatch(
            "Only variables and member expressions can be assigned values".into(),
            line,
        )),
    }
}

fn evaluate_function_body(
    name: &str,
    args: &[Expr],
    params: &[String],
    body: &[Stmt],
    local_env: &Rc<RefCell<Environment>>,
    index: usize,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let callable = ["function", "method", "constructor"];

    if args.len() != params.len() {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected {}, found {} arguments provided to {} {}",
                args.len(),
                params.len(),
                callable[index],
                name
            ),
            line,
        ));
    }

    for i in 0..args.len() {
        let value = evaluate_expr(&args[i], local_env)?;
        if let Err(_) = declare_var(&local_env, &params[i][..], value, false) {
            return Err(RuntimeError::EnvironmentError(
                format!(
                    "{} is already declared. Cannot redeclare variable with same name",
                    params[i]
                ),
                line,
            ));
        }
    }

    for stmt in body {
        match evaluate(&stmt, local_env)? {
            EvalResult::Return(val) => return Ok(val),
            _ => continue,
        }
    }

    Ok(make_nil())
}

fn evaluate_function_call(
    args: &[Expr],
    caller: &Expr,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let call = evaluate_expr(caller, env)?;
    match call {
        RuntimeVal::Class { name, methods, .. } => {
            let instance_env = Environment::new(None);
            let class_constructor = methods.get(name.as_str());
            let instance = make_instance(&name[..], instance_env);
            if let Some(func) = class_constructor {
                if let RuntimeVal::Function {
                    name,
                    params,
                    body,
                    closure,
                } = func
                {
                    let local_env = Environment::new(Some(Rc::clone(&closure)));
                    if let Err(_) = declare_var(&local_env, "this", instance.clone(), false) {
                        return Err(RuntimeError::InternalError);
                    }
                    let _ = evaluate_function_body(
                        &name[..],
                        args,
                        &params,
                        &body,
                        &local_env,
                        2,
                        line,
                    )?;
                }
            }
            Ok(instance)
        }

        RuntimeVal::Method { name, params, body, closure, instance } => {
            let local_env = Environment::new(Some(Rc::clone(&closure)));
            if let Err(_) = declare_var(&local_env, "this", *instance, true) {
                return Err(RuntimeError::InternalError);
            }
            evaluate_function_body(
                &name[..],
                args,
                &params,
                &body,
                &local_env,
                1,
                line,
            )
        }

        RuntimeVal::Function {
            name,
            params,
            body,
            closure,
        } => {
            let local_env = Environment::new(Some(Rc::clone(&closure)));
            evaluate_function_body(&name[..], args, &params, &body, &local_env, 0, line)
        }

        RuntimeVal::NativeFunction(func, ..) => {
            let mut values = vec![];
            for arg in args {
                values.push(evaluate_expr(&arg, env)?);
            }
            func(&values, line)
        }
        _ => Err(RuntimeError::InvalidCall("Expected function, method or class type for call expression".to_string(), line))
    }
}

fn evaluate_member_expr(
    object: &Expr,
    property: &Expr,
    computed: bool,
    env: &Rc<RefCell<Environment>>,
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let mut obj = evaluate_expr(object, env)?;

    if computed {
        let key = evaluate_expr(property, env)?;
        match (obj, key) {
            (RuntimeVal::Object(map), RuntimeVal::String(str)) => {
                let value = map.get(str.as_str());
                match value {
                    Some(val) => Ok(val.clone()),
                    None => Ok(make_nil()),
                }
            }

            (RuntimeVal::String(str), RuntimeVal::Number(num)) => {
                if num < 0.0 || num.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", num), line));
                }
                let pos_num = num as usize;
                if pos_num >= str.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                Ok(make_string(&str.chars().nth(pos_num).unwrap().to_string()[..]))
            }

            (RuntimeVal::Array(arr), RuntimeVal::Number(num)) => {
                if num < 0.0 || num.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", num), line));
                }
                let pos_num = num as usize;
                if pos_num >= arr.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                Ok(arr[pos_num].clone())
            }

            _ => Err(RuntimeError::InvalidMemberAccess("[]".into(), line)),
        }
    } else {
        let lexeme = match property {
            Expr::Identifier(name, _) => name,
            _ => return Err(RuntimeError::InternalError),
        };
        let mut method_exists = None;
        loop {
            match obj {
                RuntimeVal::Object(map) => {
                    let res = map.get(lexeme.as_str());
                    return match res {
                        Some(value) => Ok(value.clone()),
                        None => {
                            Err(RuntimeError::UndefinedField(
                                format!("Object has no field named '{}'", lexeme),
                                line,
                            ))
                        }
                    }
                }

                RuntimeVal::Class {
                    name,
                    static_fields,
                    methods,
                    superclass,
                    ..
                } => {
                    let method = methods.get(lexeme);
                    if let Some(method) = method {
                        if let Some(val) = method_exists {
                            if let RuntimeVal::Function {name, params, body, closure} = method {
                                return Ok(make_method(name, params, body, closure, val));
                            }
                        }
                        return Ok(method.clone());
                    }
                    let static_field = static_fields.get(lexeme);
                    if let Some(static_field) = static_field {
                        return Ok(static_field.clone());
                    }

                    match superclass {
                        Some(parent) => {
                            obj = match lookup_var(env, &parent[..]) {
                                Ok(val) => val,
                                Err(_) => {
                                    return Err(RuntimeError::EnvironmentError(
                                        format!(
                                            "'{}' superclass is not defined but is inherited by class '{}'.",
                                            parent, name
                                        ),
                                        line,
                                    ));
                                }
                            };
                        }
                        None => {
                            return Err(RuntimeError::UndefinedProperty(
                                format!(
                                    "Property '{}' is not defined in class '{}' or superclasses",
                                    lexeme, name
                                ),
                                line,
                            ));
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
                        Err(_) => return Err(RuntimeError::InternalError),
                    },
                },

                _ => return Err(RuntimeError::InvalidMemberAccess(".".into(), line)),
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
    line: usize,
) -> Result<RuntimeVal, RuntimeError> {
    let result = evaluate_expr(value, env)?;
    let obj = evaluate_expr(object, env)?;
    let lexeme_name = match object {
        Expr::Identifier(s, _) => s,
        _ => return Err(RuntimeError::InternalError),
    };

    if computed {
        let key = evaluate_expr(property, env)?;
        match (obj, key) {
            (RuntimeVal::Object(mut map), RuntimeVal::String(str)) => {
                map.insert(str, result.clone());
                let val = make_obj(&map);
                if let Err(_) = assign_var(env, &lexeme_name[..], val) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "'{}' is a constant. Constant values cannot be reassigned.",
                            lexeme_name
                        ),
                        line,
                    ));
                }
            }

            (RuntimeVal::String(str), RuntimeVal::Number(num)) => {
                if num < 0.0 || num.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", num), line));
                }
                let pos_num = num as usize;
                if pos_num >= str.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                let res = match result {
                    RuntimeVal::String(ref s) => s,
                    _ => return Err(RuntimeError::TypeMismatch("Cannot assign non-string type value to string index".to_string(), line))
                };
                let new_str = format!("{}{}{}", &str[..pos_num], res, &str[pos_num+1..]);
                let val = make_string(&new_str);
                if let Err(_) = assign_var(env, &lexeme_name[..], val) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "'{}' is a constant. Constant values cannot be reassigned.",
                            lexeme_name
                        ),
                        line,
                    ));
                }
            }

            (RuntimeVal::Array(mut arr), RuntimeVal::Number(num)) => {
                if num < 0.0 || num.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", num), line));
                }
                let pos_num = num as usize;
                if pos_num >= arr.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                arr[pos_num] = result.clone();
                let val = make_arr(&arr);
                if let Err(_) = assign_var(env, &lexeme_name[..], val) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "'{}' is a constant. Constant values cannot be reassigned.",
                            lexeme_name
                        ),
                        line,
                    ));
                }
            }

            _ => return Err(RuntimeError::InvalidMemberAccess("[]".into(), line)),
        }
    } else {
        let lexeme = match property {
            Expr::Identifier(name, _) => name,
            _ => return Err(RuntimeError::InternalError),
        };
        match obj {
            RuntimeVal::Object(mut map) => {
                map.insert(lexeme.clone(), result.clone());
                let val = make_obj(&map);
                if let Err(_) = assign_var(env, &lexeme_name[..], val) {
                    return Err(RuntimeError::EnvironmentError(
                        format!(
                            "'{}' is a constant. Constant values cannot be reassigned.",
                            lexeme_name
                        ),
                        line,
                    ));
                }
            }

            RuntimeVal::Class {
                name,
                mut static_fields,
                methods,
                superclass,
            } => {
                let method = methods.get(lexeme);
                if let Some(_) = method {
                    return Err(RuntimeError::TypeMismatch(
                        format!(
                            "Cannot assign value to method '{}' of class '{}'",
                            lexeme, name
                        ),
                        line,
                    ));
                }
                static_fields.insert(lexeme.clone(), result.clone());
                let val = make_class(&name, static_fields, methods, superclass);
                if let Err(_) = assign_var(env, &name[..], val) {
                    return Err(RuntimeError::InternalError);
                }
            }

            RuntimeVal::Instance { instance_env, .. } => {
                if let Err(_) = declare_var(&instance_env, &lexeme[..], result.clone(), false) {
                    if let Err(_) = assign_var(&instance_env, &lexeme[..], result.clone()) {
                        return Err(RuntimeError::InternalError);
                    }
                }
            }

            _ => return Err(RuntimeError::InvalidMemberAccess(".".into(), line)),
        }
    }
    Ok(result)
}
