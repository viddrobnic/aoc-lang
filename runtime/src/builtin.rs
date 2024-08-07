use std::{fmt::Display, io, rc::Rc};

use crate::{
    error::ErrorKind,
    object::{self, Array, Dictionary, HashKey, Object},
    vm::gc::GarbageCollector,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Builtin {
    Len,

    Str,
    Int,
    Char,
    Float,
    Bool,
    IsNull,

    Floor,
    Ceil,
    Round,

    TrimStart,
    TrimEnd,
    Trim,
    Split,

    Push,
    Pop,
    Del,

    Print,
    Input,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Len => write!(f, "len"),
            Builtin::Str => write!(f, "str"),
            Builtin::Int => write!(f, "int"),
            Builtin::Char => write!(f, "char"),
            Builtin::Float => write!(f, "float"),
            Builtin::Bool => write!(f, "bool"),
            Builtin::IsNull => write!(f, "is_null"),
            Builtin::Floor => write!(f, "floor"),
            Builtin::Ceil => write!(f, "ceil"),
            Builtin::Round => write!(f, "round"),
            Builtin::TrimStart => write!(f, "trim_start"),
            Builtin::TrimEnd => write!(f, "trim_end"),
            Builtin::Trim => write!(f, "trim"),
            Builtin::Split => write!(f, "split"),
            Builtin::Push => write!(f, "push"),
            Builtin::Pop => write!(f, "pop"),
            Builtin::Del => write!(f, "del"),
            Builtin::Print => write!(f, "print"),
            Builtin::Input => write!(f, "input"),
        }
    }
}

impl Builtin {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let builtin = match ident {
            "len" => Self::Len,
            "str" => Self::Str,
            "int" => Self::Int,
            "char" => Self::Char,
            "float" => Self::Float,
            "bool" => Self::Bool,
            "is_null" => Self::IsNull,
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "round" => Self::Round,
            "trim_start" => Self::TrimStart,
            "trim_end" => Self::TrimEnd,
            "trim" => Self::Trim,
            "split" => Self::Split,
            "push" => Self::Push,
            "pop" => Self::Pop,
            "del" => Self::Del,
            "print" => Self::Print,
            "input" => Self::Input,

            _ => return None,
        };

        Some(builtin)
    }

    pub fn documentation(&self) -> String {
        let doc = match self {
            Builtin::Len => {
                r#"
Returns length of the parameter. Parameter can be string, array or dictionary.

Usage:
```aoc
len([1, 2, 3])  // 3
```
                "#
            }
            Builtin::Str => {
                r#"
Returns string representation of the parameter. Paremeter can be int, bool, float or char.

Usage:
```aoc
str(10)    // "10"
str(4.2)   // "4.2"
str(true)  // "true"
str(false) // "false"
str('a')   // "a"
```
                "#
            }
            Builtin::Int => {
                r#"
Converts input to integer. Parameter can be float, string or char.

Usage:
```aoc
int(4.2)  // 4
int("12") // 12
int('a')  // 97
```
                "#
            }
            Builtin::Char => {
                r#"
Converts int to char.

Usage:
```aoc
char(97) // 'a'
```
                "#
            }
            Builtin::Float => {
                r#"
Converts parameter to float. Parameter can be int or string.

Usage:
```aoc
float(4)     // 4.0
float("4.2") // 4.2
```
                "#
            }
            Builtin::Bool => {
                r#"
Returns weather the parameter is truthy. Everything except
`false` and `null` is truthy.

Usage:
```aoc
bool(false) // false
bool(null)  // false
bool("")    // true
```
                "#
            }
            Builtin::IsNull => {
                r#"
Returns if parameter is null.

Usage:
```aoc
is_null(null)  // true
is_null(false) // false
```
                "#
            }
            Builtin::Floor => {
                r#"
Rounds down given float to the nearest integer.

Usage:
```aoc
floor(4.9) // 4.0
```
                "#
            }
            Builtin::Ceil => {
                r#"
Rounds up given float to the nearest integer.

Usage:
```aoc
ceil(4.1) // 5.0
```
                "#
            }
            Builtin::Round => {
                r#"
Rounds given float to the nearest integer.

Usage:
```aoc
round(4.2) // 4.0
round(4.5) // 5.0
round(4.8) // 5.0
```
                "#
            }
            Builtin::TrimStart => {
                r#"
Removes leading whitespace from string.

Usage:
```aoc
trim_start("  foo\n") // "foo\n"
```
                "#
            }
            Builtin::TrimEnd => {
                r#"
Removes trailing whitespace from string.

Usage:
```aoc
trim_end("  foo\n") // "  foo"
```
                "#
            }
            Builtin::Trim => {
                r#"
Removes leading and trailing whitespace from string.

Usage:
```aoc
trim("  foo\n") // "foo"
```
                "#
            }
            Builtin::Split => {
                r#"
Splits the given string by a given delimeter.

Usage:
```aoc
split("foo", "")      // ["f", "o", "o"]
split("foo bar", " ") // ["foo", "bar"]
```
                "#
            }
            Builtin::Push => {
                r#"
Adds element to the end of the array. Given array is mutated.

Usage:
```aoc
arr = []
push(arr, 1) // null
arr          // [1]
```
                "#
            }
            Builtin::Pop => {
                r#"
Pops last element of the array and returns it. Given array is mutated.
If the array is empty, function returns `null`.

Usage:
```aoc
arr = [1]
pop(arr) // 1
arr      // []
pop(arr) // null
```
                "#
            }
            Builtin::Del => {
                r#"
Deletes entry with given key in the dictionary. Deleted entry is returned.
If no entry under the key exists, `null` is returned.

Usage:
```aoc
dict = { "foo": "bar" }
del(dict, "foo") // "bar"
dict             // {}
del(dict, "bar") // null
```
                "#
            }
            Builtin::Print => {
                r#"
Prints parameter to stdout. Parameter can be null, int, float, bool, string or char.

Usage:
```aoc
print("Hello world!") // Hello world!
```
                "#
            }
            Builtin::Input => {
                r#"
Reads a single line from stdin. If EOF is reached, `null` is returned.
The returned string doesn't contain trailing '\n'.

Usage:
```aoc
input()
```
                "#
            }
        };

        doc.to_string()
    }

    pub(crate) fn call(
        &self,
        args: &[Object],
        gc: &mut GarbageCollector,
    ) -> Result<Object, ErrorKind> {
        match self {
            Builtin::Len => call_len(args),

            Builtin::Str => call_str(args),
            Builtin::Int => call_int(args),
            Builtin::Char => call_char(args),
            Builtin::Float => call_float(args),
            Builtin::Bool => call_bool(args),
            Builtin::IsNull => is_null(args),

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
            Builtin::Del => call_del(args),

            Builtin::Print => call_print(args),
            Builtin::Input => call_input(args),
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
        Object::Char(ch) => Rc::new((*ch as char).to_string()),

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
        Object::Char(ch) => *ch as i64,

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Int,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Integer(res))
}

fn call_char(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    let res = match &args[0] {
        Object::Char(ch) => *ch,
        Object::Integer(int) => *int as u8,

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Char,
                data_type: obj.into(),
            })
        }
    };

    Ok(Object::Char(res))
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

    Ok(Object::Boolean(args[0].is_truthy()))
}

fn is_null(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;
    Ok(Object::Boolean(args[0] == Object::Null))
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

fn call_del(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 2)?;

    let Object::Dictionary(Dictionary(dict)) = &args[0] else {
        return Err(ErrorKind::InvalidBuiltinArg {
            builtin: Builtin::Del,
            data_type: (&args[0]).into(),
        });
    };

    let key: HashKey = args[1].clone().try_into()?;
    let rc = dict.value.upgrade().unwrap();
    let obj = rc.borrow_mut().remove(&key);
    match obj {
        Some(obj) => Ok(obj),
        None => Ok(Object::Null),
    }
}

fn call_print(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 1)?;

    match &args[0] {
        Object::Null => println!("null"),
        Object::Integer(val) => println!("{val}"),
        Object::Float(val) => println!("{val}"),
        Object::Boolean(val) => println!("{val}"),
        Object::String(val) => println!("{val}"),
        Object::Char(ch) => println!("{}", *ch as char),

        obj => {
            return Err(ErrorKind::InvalidBuiltinArg {
                builtin: Builtin::Print,
                data_type: obj.into(),
            })
        }
    }

    Ok(Object::Null)
}

fn call_input(args: &[Object]) -> Result<Object, ErrorKind> {
    validate_args_len(args, 0)?;

    let mut line = String::new();
    let read = io::stdin()
        .read_line(&mut line)
        .map_err(|_| ErrorKind::InputError)?;

    if read == 0 {
        // Handle eof
        Ok(Object::Null)
    } else {
        // Remove \n that is always read
        line.pop();
        Ok(Object::String(Rc::new(line)))
    }
}
