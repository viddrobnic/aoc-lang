use crate::{error::ErrorKind, object::Object};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Builtin {
    Len,
}

impl Builtin {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let builtin = match ident {
            "len" => Self::Len,

            _ => return None,
        };

        Some(builtin)
    }

    pub fn call(&self, args: &[Object]) -> Result<Object, ErrorKind> {
        match self {
            Builtin::Len => call_len(args),
        }
    }
}

fn call_len(args: &[Object]) -> Result<Object, ErrorKind> {
    if args.len() != 1 {
        return Err(ErrorKind::InvalidNrOfArgs {
            expected: 1,
            got: args.len(),
        });
    }

    let res = match &args[0] {
        Object::String(str) => str.len(),
        Object::Array(arr) => arr.0.value.upgrade().unwrap().borrow().len(),
        Object::Dictionary(dict) => dict.0.value.upgrade().unwrap().borrow().len(),

        obj => return Err(ErrorKind::InvalidLengthCalle(obj.into())),
    };

    Ok(Object::Integer(res as i64))
}
