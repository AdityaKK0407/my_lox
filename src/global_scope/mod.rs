use std::io;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::handle_errors::RuntimeError;
use crate::values::*;

pub fn clock(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() > 0 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'clock'",
                args.len()
            ),
            line,
        ));
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time = now.as_secs_f64() + now.as_nanos() as f64 * 1e-9;
    Ok(make_number(time))
}

pub fn scan(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 0 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'reverse'",
                args.len()
            ),
            line,
        ));
    }

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to scan input");

    Ok(make_string(&input[..]))
}

pub fn min(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected more than 2, found {} arguments provided to native function 'min'",
                args.len()
            ),
            line,
        ));
    }

    let mut min = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => {
            return Err(RuntimeError::TypeMismatch(
                "Only type number and array allowed in 'min' function".to_string(),
                line,
            ));
        }
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > min {
                min = *num;
            }
        } else {
            return Err(RuntimeError::TypeMismatch(
                "Only type number and array allowed in 'min' function".to_string(),
                line,
            ));
        }
    }

    Ok(make_number(min))
}

pub fn max(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected more than 2, found {} arguments provided to native function 'max'",
                args.len()
            ),
            line,
        ));
    }

    let mut max = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => {
            return Err(RuntimeError::TypeMismatch(
                "Only type number and array allowed in 'max' function".to_string(),
                line,
            ));
        }
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > max {
                max = *num;
            }
        } else {
            return Err(RuntimeError::TypeMismatch(
                "Only type number and array allowed in 'min' function".to_string(),
                line,
            ));
        }
    }

    Ok(make_number(max))
}

pub fn number(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'number'",
                args.len()
            ),
            line,
        ));
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
        RuntimeVal::String(str) => match str.parse::<f64>().ok() {
            Some(n) => Ok(make_number(n)),
            None => Err(RuntimeError::TypeCastingError(
                "Invalid string provided, expected numeric string in 'number' function".to_string(),
                line,
            )),
        },
        _ => {
            Err(RuntimeError::TypeMismatch(
                "Only type number, bool and string allowed in 'number' function".to_string(),
                line,
            ))
        }
    }
}

pub fn bool(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'bool'",
                args.len()
            ),
            line,
        ));
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
            Err(RuntimeError::TypeMismatch(
                "Only type number, bool and string allowed in 'bool' function".to_string(),
                line,
            ))
        }
    }
}

pub fn string(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'string'",
                args.len()
            ),
            line,
        ));
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
            Err(RuntimeError::TypeMismatch(
                "Only type number, bool and string allowed in 'string' function".to_string(),
                line,
            ))
        }
    }
}

pub fn len(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'var_type'",
                args.len()
            ),
            line,
        ));
    }

    match &args[0] {
        RuntimeVal::String(s) => Ok(make_number(s.len() as f64)),
        RuntimeVal::Array(arr) => Ok(make_number(arr.len() as f64)),
        _ => Err(RuntimeError::TypeMismatch(
            "Only type string and array allowed in 'len' function".to_string(),
            line,
        )),
    }
}

pub fn type_of(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'var_type'",
                args.len()
            ),
            line,
        ));
    }

    match &args[0] {
        RuntimeVal::Number(_) => Ok(make_string("Number")),
        RuntimeVal::Bool(_) => Ok(make_string("Bool")),
        RuntimeVal::Nil => Ok(make_string("Nil")),
        RuntimeVal::String(_) => Ok(make_string("String")),
        RuntimeVal::Object(_) => Ok(make_string("Object")),
        RuntimeVal::Array(_) => Ok(make_string("Array")),
        RuntimeVal::Function { .. } => Ok(make_string("Function")),
        RuntimeVal::NativeFunction(_, _) => Ok(make_string("Native function")),
        RuntimeVal::Method { .. } => Ok(make_string("Method")),
        RuntimeVal::Class { .. } => Ok(make_string("Class")),
        RuntimeVal::Instance { .. } => Ok(make_string("Instance")),
    }
}

pub fn reverse(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgumentCount(
            format!(
                "Expected 1, found {} arguments provided to native function 'reverse'",
                args.len()
            ),
            line,
        ));
    }

    match &args[0] {
        RuntimeVal::String(s) => Ok(make_string(&s.chars().rev().collect::<String>()[..])),
        RuntimeVal::Array(arr) => Ok(make_arr(&arr.clone().into_iter().rev().collect())),
        _ => Err(RuntimeError::TypeMismatch(
            "Only type string allowed in 'reverse' function".to_string(),
            line,
        )),
    }
}

pub fn append(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(RuntimeError::InvalidArgumentCount(format!(
            "Expected 2 | 3, found {} arguments provided to native function 'append'",
            args.len()
        ), line));
    }
    let mut array = match &args[0] {
        RuntimeVal::Array(arr) => arr.clone(),
        _ => return Err(RuntimeError::TypeMismatch(
            "Only type array allowed as first argument in 'append' function".to_string(),
            line,
        )),
    };
    
    let val = &args[1];
    
    let position;
    if args.len() == 2 {
        position = array.len() - 1;
    } else {
        match &args[2] {
            RuntimeVal::Number(pos) => {
                if *pos < 0.0 || pos.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", pos), line));
                }
                let pos_num = *pos as usize;
                if pos_num >= array.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                position = pos_num; 
            },
            _ => return Err(RuntimeError::TypeMismatch("Only type number allowed as third argument in 'append' function".to_string(), line)),
        };
    }
    array.insert(position, val.clone());
    Ok(RuntimeVal::Array(array))
}

pub fn remove(args: &[RuntimeVal], line: usize) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 1 || args.len() > 2 {
        return Err(RuntimeError::InvalidArgumentCount(format!(
            "Expected 2 | 3, found {} arguments provided to native function 'remove'",
            args.len()
        ), line));
    }
    let mut array = match &args[0] {
        RuntimeVal::Array(arr) => arr.clone(),
        _ => return Err(RuntimeError::TypeMismatch(
            "Only type array allowed as first argument in 'remove' function".to_string(),
            line,
        )),
    };

    let position;
    if args.len() == 1 {
        position = array.len() - 1;
    } else {
        match &args[1] {
            RuntimeVal::Number(pos) => {
                if *pos < 0.0 || pos.fract() != 0.0 {
                    return Err(RuntimeError::InvalidArrayIndex(format!("'{}' is an invalid type. Arrays can only be accessed with positive integers", pos), line));
                }
                let pos_num = *pos as usize;
                if pos_num >= array.len() {
                    return Err(RuntimeError::ArrayIndexOutOfBounds("Array index is out of bounds".to_string(), line));
                }
                position = pos_num;
            },
            _ => return Err(RuntimeError::TypeMismatch("Only type number allowed as third argument in 'remove' function".to_string(), line)),
        };
    }
    array.remove(position);
    Ok(RuntimeVal::Array(array))
}
