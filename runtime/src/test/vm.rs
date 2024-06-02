use std::{cell::RefCell, collections::HashMap, rc::Rc};

use parser::position::{Position, Range};

use crate::{
    compiler::Compiler,
    error::{Error, ErrorKind},
    object::{Array, DataType, Dictionary, HashKey, Object},
    vm::{gc, VirtualMachine},
};

fn run_test(input: &str, expected: Result<Object, Error>) {
    let program = parser::parse(input).unwrap();

    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program);

    let mut vm = VirtualMachine::new();
    let obj = vm.run(&bytecode);

    assert_eq!(obj, expected);
}

#[test]
fn constants() {
    let tests = [
        ("10", Object::Integer(10)),
        ("6.9", Object::Float(6.9)),
        ("\"foo\"", Object::String(Rc::new("foo".to_string()))),
        ("true", Object::Boolean(true)),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn array() {
    let tests = [
        ("[]", vec![]),
        ("[1]", vec![Object::Integer(1)]),
        (
            "[1, \"foo\"]",
            vec![
                Object::Integer(1),
                Object::String(Rc::new("foo".to_string())),
            ],
        ),
    ];

    for (input, expected) in tests {
        let rc = Rc::new(RefCell::new(expected));
        let arr = Array(gc::Ref {
            value: Rc::downgrade(&rc),
            id: 0,
        });
        run_test(input, Ok(Object::Array(arr)));
    }
}

#[test]
fn nested_array() {
    let input = "[1, [2, 3], 4]";

    let inner = Rc::new(RefCell::new(vec![Object::Integer(2), Object::Integer(3)]));
    let inner_arr = Object::Array(Array(gc::Ref {
        value: Rc::downgrade(&inner),
        id: 0,
    }));

    let outer = Rc::new(RefCell::new(vec![
        Object::Integer(1),
        inner_arr,
        Object::Integer(4),
    ]));
    let outer_arr = Object::Array(Array(gc::Ref {
        value: Rc::downgrade(&outer),
        id: 0,
    }));

    run_test(input, Ok(outer_arr));
}

#[test]
fn hash_map() {
    let tests = [
        ("{}", HashMap::from([])),
        (
            "{\"foo\": 1}",
            HashMap::from([(
                HashKey::String(Rc::new("foo".to_string())),
                Object::Integer(1),
            )]),
        ),
        // (
        //     "{1: 2, 2: {3: 4}}",
        //     HashMap::from([
        //         (HashKey::Integer(1), Object::Integer(2)),
        //         (
        //             HashKey::Integer(2),
        //             Object::Dictionary(Rc::new(HashMap::from([(
        //                 HashKey::Integer(3),
        //                 Object::Integer(4),
        //             )]))),
        //         ),
        //     ]),
        // ),
    ];

    for (input, expected) in tests {
        let dict = Rc::new(RefCell::new(expected));
        let dict_ref = gc::Ref {
            value: Rc::downgrade(&dict),
            id: 0,
        };
        run_test(input, Ok(Object::Dictionary(Dictionary(dict_ref))));
    }
}

#[test]
fn nested_hash_map() {
    let input = "{1: 2, 2: {3: 4}}";

    let inner = Rc::new(RefCell::new(HashMap::from([(
        HashKey::Integer(3),
        Object::Integer(4),
    )])));
    let inner_ref = gc::Ref {
        value: Rc::downgrade(&inner),
        id: 0,
    };

    let outer = Rc::new(RefCell::new(HashMap::from([
        (HashKey::Integer(1), Object::Integer(2)),
        (
            HashKey::Integer(2),
            Object::Dictionary(Dictionary(inner_ref)),
        ),
    ])));
    let outer_ref = gc::Ref {
        value: Rc::downgrade(&outer),
        id: 0,
    };

    run_test(input, Ok(Object::Dictionary(Dictionary(outer_ref))));
}

#[test]
fn hash_map_error() {
    let tests = [(
        "{1.9: true}",
        Error {
            kind: ErrorKind::NotHashable(DataType::Float),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 11),
            },
        },
    )];

    for (input, expected) in tests {
        run_test(input, Err(expected));
    }
}

#[test]
fn prefix_operator() {
    let tests = [
        ("-10", Object::Integer(-10)),
        ("-4.2", Object::Float(-4.2)),
        ("--10", Object::Integer(10)),
        ("-(-10)", Object::Integer(10)),
        ("!false", Object::Boolean(true)),
        ("!!false", Object::Boolean(false)),
        ("!42", Object::Integer(-43)), // two's complement
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}
