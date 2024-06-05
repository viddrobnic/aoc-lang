use parser::position::{Position, Range};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    compiler::Compiler,
    error::{Error, ErrorKind},
    object::{Array, Closure, DataType, Dictionary, HashKey, Object},
    vm::{gc, VirtualMachine},
};

fn run_test(input: &str, expected: Result<Object, Error>) {
    let program = parser::parse(input).unwrap();

    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program).unwrap();

    let mut vm = VirtualMachine::new();
    let vm_res = vm.run(&bytecode);

    // We check the first element on the stack. If compiler and vm both
    // work correctly, the vm should get cleaned up and the first element on
    // the stack should be the last popped element.
    let obj = vm.stack[0].clone();
    let res = match vm_res {
        Ok(_) => Ok(obj),
        Err(err) => Err(err),
    };

    assert_eq!(res, expected);
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

#[test]
fn while_loop() {
    let tests = [
        ("while (false) {}\n 10", Object::Integer(10)),
        (
            "i = 0\n while (i < 10) {i = i + 1}\n i",
            Object::Integer(10),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn assign() {
    let tests = [
        ("a = 10\n a", Object::Integer(10)),
        ("[a, b] = [1, 2]\n a", Object::Integer(1)),
        ("[a, b] = [1, 2]\n b", Object::Integer(2)),
        ("[a, [b, c]] = [1, [2, 3]]\n a", Object::Integer(1)),
        ("[a, [b, c]] = [1, [2, 3]]\n b", Object::Integer(2)),
        ("[a, [b, c]] = [1, [2, 3]]\n c", Object::Integer(3)),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn assign_array_index() {
    let tests = [
        ("a = [0]\n a[0] = 1\n a", vec![Object::Integer(1)]),
        (
            "a = [0, 1]\n [a[0], a[1]] = [42, 69]\n a",
            vec![Object::Integer(42), Object::Integer(69)],
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
fn assign_dict_index() {
    let tests = [
        (
            "a = {}\n a[0] = 1\n a",
            HashMap::from([(HashKey::Integer(0), Object::Integer(1))]),
        ),
        (
            "a = {}\n a.foo = 42\n a",
            HashMap::from([(
                HashKey::String(Rc::new("foo".to_string())),
                Object::Integer(42),
            )]),
        ),
        (
            "a = {}\n [a.foo, a[\"bar\"]] = [42, 69]\n a",
            HashMap::from([
                (
                    HashKey::String(Rc::new("foo".to_string())),
                    Object::Integer(42),
                ),
                (
                    HashKey::String(Rc::new("bar".to_string())),
                    Object::Integer(69),
                ),
            ]),
        ),
    ];

    for (input, expected) in tests {
        let rc = Rc::new(RefCell::new(expected));
        let arr = Dictionary(gc::Ref {
            value: Rc::downgrade(&rc),
            id: 0,
        });
        run_test(input, Ok(Object::Dictionary(arr)));
    }
}
#[test]

fn infix_operator() {
    let tests = [
        ("1 + 1", Object::Integer(2)),
        ("1 + 2 * 3", Object::Integer(7)),
        ("4.2 * 2.0", Object::Float(8.4)),
        ("3 - 2 - 1", Object::Integer(0)),
        (
            "\"foo\" + \" \" + \"bar\"",
            Object::String(Rc::new("foo bar".to_string())),
        ),
        ("3 % 2", Object::Integer(1)),
        ("-3 % 2", Object::Integer(1)),
        ("0 & 1", Object::Integer(0)),
        ("1 | 1", Object::Integer(1)),
        ("true & false", Object::Boolean(false)),
        ("1 < 2 & 3 < 4 | 5 == 2", Object::Boolean(true)),
        ("\"abc\" < \"aab\"", Object::Boolean(false)),
        ("\"abc\" == \"abc\"", Object::Boolean(true)),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn index() {
    let tests = [
        ("[1, 2, 3][0]", Object::Integer(1)),
        ("[1, 2, 3][1]", Object::Integer(2)),
        ("[][0]", Object::Null),
        ("[][-10]", Object::Null),
        ("foo = []\n foo[0]", Object::Null),
        (
            "foo = [\"bar\"]\n foo[0]",
            Object::String(Rc::new("bar".to_string())),
        ),
        ("{true: false}[true]", Object::Boolean(false)),
        ("{\"foo\": 10}.foo", Object::Integer(10)),
        ("{\"foo\": 10}[\"foo\"]", Object::Integer(10)),
        (
            "foo = {\"property\": {\"nested\": 42}}\n foo.property.nested",
            Object::Integer(42),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn for_loop() {
    let input = "for (i = 0; i < 42; i = i + 1) {}\n i";
    run_test(input, Ok(Object::Integer(42)));
}

#[test]
fn if_statement() {
    let tests = [
        ("if (1 < 2) {10}", Object::Integer(10)),
        ("if (1 > 2) {10}", Object::Null),
        ("if (true) {10} else {}", Object::Integer(10)),
        ("if (false) {10} else {20}", Object::Integer(20)),
        ("if (false) {10} else if (false) {20}", Object::Null),
        (
            "if (false) {10} else if (false) {20} else {30}",
            Object::Integer(30),
        ),
        (
            "if (false) {10} else if (false) {20} else if (true) {30} else {40}",
            Object::Integer(30),
        ),
        (
            r#"
            if (1 * 2 * 3 - 5 == 1) {
                a = 10
                a = a * 6
                a + 9
            } else {
                42
            }
            "#,
            Object::Integer(69),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn loop_break() {
    let tests = [
        ("while (true) {foo = 0\n break}\n foo", Object::Integer(0)),
        (
            "for (i = 0; i < 10; i = i + 1) {break}\n i",
            Object::Integer(0),
        ),
        (
            r#"
            for (i = 0; i < 100; i = i + 1) {
                if (i == 42) {
                    break
                }
            }
            i
            "#,
            Object::Integer(42),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn loop_continue() {
    let tests = [
        (
            r#"
            foo = 69
            i = 0
            while (i < 1) {
                i = i + 1
                continue
                foo = 50
            }
            foo
            "#,
            Object::Integer(69),
        ),
        (
            r#"
            sum = 0
            for (i = 0; i < 10; i = i + 1) {
                if (i <= 1) {
                    continue
                }

                sum = sum + i
            }
            sum
            "#,
            Object::Integer(44),
        ),
    ];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}

#[test]
fn closure() {
    let tests = [(
        "fn(){}",
        Object::Closure(Closure {
            function_index: 0,
            free_variables: Rc::new(vec![]),
        }),
    )];

    for (input, expected) in tests {
        run_test(input, Ok(expected));
    }
}
