use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::Compiler,
    object::{HashKey, Object},
    vm::VirtualMachine,
};

fn run_test(input: &str, expected: Object) {
    let program = parser::parse(input).unwrap();

    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program);

    let vm = VirtualMachine::new();
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
        run_test(input, expected);
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
        (
            "[1, [2, 3], 4]",
            vec![
                Object::Integer(1),
                Object::Array(Rc::new(vec![Object::Integer(2), Object::Integer(3)])),
                Object::Integer(4),
            ],
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Object::Array(Rc::new(expected)));
    }
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
        (
            "{1: 2, 2: {3: 4}}",
            HashMap::from([
                (HashKey::Integer(1), Object::Integer(2)),
                (
                    HashKey::Integer(2),
                    Object::HashMap(Rc::new(HashMap::from([(
                        HashKey::Integer(3),
                        Object::Integer(4),
                    )]))),
                ),
            ]),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Object::HashMap(Rc::new(expected)));
    }
}
