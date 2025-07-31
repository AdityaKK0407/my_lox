use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::handle_errors::RuntimeError;
use crate::values::*;

pub fn clock(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() > 0 {
        return Err(RuntimeError::MoreFuncArguments);
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time = now.as_secs_f64() + now.as_nanos() as f64 * 1e-9;
    Ok(make_number(time))
}

pub fn min(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::LessFuncArguments);
    }

    let mut min = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => return Err(RuntimeError::MisMatchTypes),
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > min {
                min = *num;
            }
        } else {
            return Err(RuntimeError::MisMatchTypes);
        }
    }

    Ok(make_number(min))
}

pub fn max(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::LessFuncArguments);
    }

    let mut max = match &args[0] {
        RuntimeVal::Number(num) => *num,
        _ => return Err(RuntimeError::MisMatchTypes),
    };

    for arg in &args[1..] {
        if let RuntimeVal::Number(num) = arg {
            if *num > max {
                max = *num;
            }
        } else {
            return Err(RuntimeError::MisMatchTypes);
        }
    }

    Ok(make_number(max))
}

pub fn number(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::LessFuncArguments);
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
        RuntimeVal::Nil => panic!("Cannot convert type nil to type number"),
        RuntimeVal::String(str) => Ok(make_number(str.parse::<f64>().unwrap())),
        _ => return Err(RuntimeError::MisMatchTypes),
    }
}

pub fn bool(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::MoreFuncArguments);
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
        RuntimeVal::Nil => panic!("Cannot convert type nil to type bool"),
        RuntimeVal::String(str) => {
            if str.len() == 0 {
                Ok(make_bool(false))
            } else {
                Ok(make_bool(true))
            }
        }
        _ => return Err(RuntimeError::MisMatchTypes),
    }
}

pub fn string(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::MoreFuncArguments);
    }

    match args[0] {
        RuntimeVal::Number(num) => {
            let s = num.to_string();
            Ok(make_string(Box::leak(s.into_boxed_str())))
        },
        RuntimeVal::Bool(bit) => {
            if bit {
                Ok(make_string("true"))
            } else {
                Ok(make_string("false"))
            }
        }
        RuntimeVal::Nil => panic!("Cannot convert type nil to type string"),
        RuntimeVal::String(str) => Ok(make_string(str)),
        _ => return Err(RuntimeError::MisMatchTypes),
    }
}

pub fn var_type(args: Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::MoreFuncArguments);
    }

    match &args[0] {
        RuntimeVal::Number(_) => Ok(make_string("Number")),
        RuntimeVal::Bool(_) => Ok(make_string("Bool")),
        RuntimeVal::Nil => Ok(make_string("Nil")),
        RuntimeVal::String(_) => Ok(make_string("String")),
        RuntimeVal::Object(_) => Ok(make_string("<Object>")),
        _ => panic!(),
    }
}