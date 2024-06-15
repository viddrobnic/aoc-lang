use std::{fmt::Display, rc::Rc};

use crate::{
    error::ErrorKind,
    object::{self, Array, Object},
    vm::gc::GarbageCollector,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Builtin {
    Len,

    Str,
    Int,
    Float,
    Bool,

    Floor,
    Ceil,
    Round,

    TrimStart,
    TrimEnd,
    Trim,
    Split,

    Push,
    Pop,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Len => write!(f, "len"),
            Builtin::Str => write!(f, "str"),
            Builtin::Int => write!(f, "int"),
            Builtin::Float => write!(f, "float"),
            Builtin::Bool => write!(f, "bool"),
            Builtin::Floor => write!(f, "floor"),
            Builtin::Ceil => write!(f, "ceil"),
            Builtin::Round => write!(f, "round"),
            Builtin::TrimStart => write!(f, "trim_start"),
            Builtin::TrimEnd => write!(f, "trim_end"),
            Builtin::Trim => write!(f, "trim"),
            Builtin::Split => write!(f, "split"),
            Builtin::Push => write!(f, "push"),
            Builtin::Pop => write!(f, "pop"),
        }
    }
}

impl Builtin {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let builtin = match ident {
            "len" => Self::Len,
            "str" => Self::Str,
            "int" => Self::Int,
            "float" => Self::Float,
            "bool" => Self::Bool,
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "round" => Self::Round,
            "trim_start" => Self::TrimStart,
            "trim_end" => Self::TrimEnd,
            "trim" => Self::Trim,
            "split" => Self::Split,
            "push" => Self::Push,
            "pop" => Self::Pop,

            _ => return None,
        };

        Some(builtin)
    }

    pub fn call(&self, args: &[Object], gc: &mut GarbageCollector) -> Result<Object, ErrorKind> {
        match self {
            Builtin::Len => call_len(args),

            Builtin::Str => call_str(args),
            Builtin::Int => call_int(args),
            Builtin::Float => call_float(args),
            Builtin::Bool => call_bool(args),

            Builtin::Floor => call_round(args, |f| f.floor(), Builtin::Floor),
            Builtin::Ceil => call_round(args, |f| f.ceil(), Builtin::Ceil),
            Builtin::Round => call_round(args, |f| f.round(), Builtin::Round),

            Builtin::TrimStart => {
                str_transform(args, |s| s.trim_start().to_string(), Builtin::TrimStart)
            }
            Builtin::TrimEnd => str_transform(args, |s| s.trim_end().to_string(), Builtin::TrimEnd),
            Builtin::Trim => str_transform(args, |s| s.trim().to_string(), Builtin::Trim),
            Builtin::Split => call_split(args, gc),

            Builtin::Push => call_push(args),
            Builtin::Pop => call_pop(args),
        }
    }
}

fn validate_args_len(args: &[Object], expected: usize) -> Result<(), ErrorKind> {
    if args.len() != expected {
        Err(ErrorKind::InvalidNrOfArgs {
            expected,
            got: args.len(),
        })
    } else {
        Ok(())
    }
}

fn call_len(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::String(str) => str.len(),
        Object::Array(arr) => arr.0.value.upgrade().unwrap().borrow().len(),
        Object::Dictionary(dict) => dict.0.value.upgrade().unwrap().borrow().len(),

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Len,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Integer(res as i64))
}

fn call_str(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::String(str) => str.clone(),
        Object::Integer(int) => Rc::new(int.to_string()),
        Object::Boolean(boolean) => Rc::new(boolean.to_string()),
        Object::Float(float) => Rc::new(float.to_string()),

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Str,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::String(res))
}

fn call_int(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::Integer(int) => *int,
        Object::Float(flt) => *flt as i64,
        Object::String(str) => match str.parse() {
            Ok(res) => res,
            Err(_) => return Ok(Object::Null),
        },

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Int,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Integer(res))
}

fn call_float(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::Float(flt) => *flt,
        Object::Integer(int) => *int as f64,
        Object::String(str) => match str.parse() {
            Ok(res) => res,
            Err(_) => return Ok(Object::Null),
        },

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Float,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Float(res))
}

fn call_bool(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::String(str) => match str.parse() {
            Ok(res) => res,
            Err(_) => return Ok(Object::Null),
        },

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Bool,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Boolean(res))
}

fn call_round<F>(args: &[Object], round: F, builtin: Builtin) -> Result<Object, ErrorKind>
where
    F: Fn(f64) -> f64,
{
    validate_args_len(args, 1)?;

    let Object::Float(flt) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin,
            data_type: (&args[0]).into(),
        });
    };

    Ok(Object::Float(round(*flt)))
}

fn str_transform<F>(args: &[Object], transform: F, builtin: Builtin) -> Result<Object, ErrorKind>
where
    F: Fn(&str) -> String,
{
    validate_args_len(args, 1)?;

    let Object::String(string) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin,
            data_type: (&args[0]).into(),
        });
    };

    Ok(Object::String(Rc::new(transform(string))))
}

fn call_split(args: &[Object], gc: &mut GarbageCollector) -> Result<Object, ErrorKind> {
    validate_args_len(args, 2)?;

    let Object::String(string) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin: Builtin::Split,
            data_type: (&args[0]).into(),
        });
    };

    let Object::String(delimeter) = &args[1] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin: Builtin::Split,
            data_type: (&args[1]).into(),
        });
    };

    let mut parts: Vec<_> = string
        .split(delimeter.as_ref())
        .map(|s| Object::String(Rc::new(s.to_string())))
        .collect();

    if delimeter.as_ref() == "" {
        parts.remove(0);
        parts.pop();
    }

    let res = gc.allocate(parts);
    Ok(Object::Array(object::Array(res)))
}

fn call_push(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 2)?;

    let Object::Array(Array(arr)) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin: Builtin::Push,
            data_type: (&args[0]).into(),
        });
    };

    let rc = arr.value.upgrade().unwrap();
    rc.borrow_mut().push(args[1].clone());

    Ok(Object::Null)
}

fn call_pop(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let Object::Array(Array(arr)) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin: Builtin::Pop,
            data_type: (&args[0]).into(),
        });
    };

    let rc = arr.value.upgrade().unwrap();
    let obj = rc.borrow_mut().pop();
    match obj {
        Some(obj) => Ok(obj),
        None => Ok(Object::Null),
    }
}
