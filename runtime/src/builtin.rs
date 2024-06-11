use std::{fmt::Display, rc::Rc};

use crate::{error::ErrorKind, object::Object};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Builtin {
    Len,

    Str,
    Int,
    Float,

    Floor,
    Ceil,
    Round,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Len => write!(f, "len"),
            Builtin::Str => write!(f, "str"),
            Builtin::Int => write!(f, "int"),
            Builtin::Float => write!(f, "float"),
            Builtin::Floor => write!(f, "floor"),
            Builtin::Ceil => write!(f, "ceil"),
            Builtin::Round => write!(f, "round"),
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
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "round" => Self::Round,

            _ => return None,
        };

        Some(builtin)
    }

    pub fn call(&self, args: &[Object]) -> Result<Object, ErrorKind> {
        match self {
            Builtin::Len => call_len(args),
            Builtin::Str => call_str(args),
            Builtin::Int => call_int(args),
            Builtin::Float => call_float(args),
            Builtin::Floor => call_round(args, |f| f.floor(), Builtin::Floor),
            Builtin::Ceil => call_round(args, |f| f.ceil(), Builtin::Ceil),
            Builtin::Round => call_round(args, |f| f.round(), Builtin::Round),
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
