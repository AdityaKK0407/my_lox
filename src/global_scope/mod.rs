use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::handle_errors::RuntimeError;
use crate::values::*;

pub fn clock(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() > 0 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'clock'", args.len()), line));
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time = now.as_secs_f64() + now.as_nanos() as f64 * 1e-9;
    Ok(make_number(time))
}

pub fn min(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected more than 2, found {} arguments provided to native function 'min'", args.len()), line));
    }

    let mut min = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Only type number allowed in 'min' function"
            ), line));
        }
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > min {
                min = *num;
            }
        } else {
            return Err(RuntimeError::TypeMismatch(format!("Only type number allowed in 'min' function"), line));
        }
    }

    Ok(make_number(min))
}

pub fn max(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected more than 2, found {} arguments provided to native function 'max'", args.len()), line));
    }

    let mut max = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Only type number allowed in 'max' function"
            ), line));
        }
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > max {
                max = *num;
            }
        } else {
            return Err(RuntimeError::TypeMismatch(format!("Only type number allowed in 'min' function"), line));
        }
    }

    Ok(make_number(max))
}

pub fn number(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'number'", args.len()), line));
    }

    match &args[0] {
        RuntimeVal::Number(num) => Ok(make_number(*num)),
        RuntimeVal::Bool(bit) => {
            if *bit {
                Ok(make_number(1.0))
            } else {
                Ok(make_number(0.0))
            }
        }
        RuntimeVal::String(str) => Ok(make_number(str.parse::<f64>().unwrap())),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Only type number, bool and string allowed in 'number' function"
            ), line));
        }
    }
}

pub fn bool(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'bool'", args.len()), line));
    }

    match &args[0] {
        RuntimeVal::Number(num) => {
            if *num == 0.0 {
                Ok(make_bool(false))
            } else {
                Ok(make_bool(true))
            }
        }
        RuntimeVal::Bool(bit) => Ok(make_bool(*bit)),
        RuntimeVal::String(str) => {
            if str.len() == 0 {
                Ok(make_bool(false))
            } else {
                Ok(make_bool(true))
            }
        }
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Only type number, bool and string allowed in 'bool' function"
            ), line));
        }
    }
}

pub fn string(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'string'", args.len()), line));
    }

    match &args[0] {
        RuntimeVal::Number(num) => Ok(make_string(&num.to_string()[..])),
        RuntimeVal::Bool(bit) => {
            if *bit {
                Ok(make_string("true"))
            } else {
                Ok(make_string("false"))
            }
        }
        RuntimeVal::String(str) => Ok(make_string(&str[..])),
        _ => {
            return Err(RuntimeError::TypeMismatch(format!(
                "Only type number, bool and string allowed in 'string' function"
            ), line));
        }
    }
}

pub fn var_type(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'var_type'", args.len()), line));
    }

    match &args[0] {
        RuntimeVal::Number(_) => Ok(make_string("Number")),
        RuntimeVal::Bool(_) => Ok(make_string("Bool")),
        RuntimeVal::Nil => Ok(make_string("Nil")),
        RuntimeVal::String(_) => Ok(make_string("String")),
        RuntimeVal::Object(_) => Ok(make_string("Object")),
        RuntimeVal::Function { .. } => Ok(make_string("Function")),
        RuntimeVal::NativeFunction(_) => Ok(make_string("Native function")),
        RuntimeVal::Method { .. } => Ok(make_string("Method")),
        RuntimeVal::Class { .. } => Ok(make_string("Class")),
        RuntimeVal::Instance { .. } => Ok(make_string("Instance")),
    }
}

pub fn reverse(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(format!("Expected 1, found {} arguments provided to native function 'reverse'", args.len()), line));
    }

    match &args[0] {
        RuntimeVal::String(s) => Ok(make_string(&s.chars().rev().collect::<String>()[..])),
        _ => Err(RuntimeError::TypeMismatch(format!("Only type string allowed in 'reverse' function"), line)),
    }
}
