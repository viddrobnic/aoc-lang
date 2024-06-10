use std::rc::Rc;

use parser::{
    parse,
    position::{Position, Range},
};

use crate::{
    builtin::Builtin,
    bytecode::{Bytecode, CreateClosure, Function, Instruction},
    compiler::Compiler,
    object::Object,
};

#[test]
fn constants() {
    let tests = [
        (
            "420",
            Bytecode {
                constants: vec![Object::Integer(420)],
                functions: vec![Function {
                    instructions: vec![Instruction::Constant(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3)
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "4.2",
            Bytecode {
                constants: vec![Object::Float(4.2)],
                functions: vec![Function {
                    instructions: vec![Instruction::Constant(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3)
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "true",
            Bytecode {
                constants: vec![Object::Boolean(true)],
                functions: vec![Function {
                    instructions: vec![Instruction::Constant(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 4)
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "\"foo\"",
            Bytecode {
                constants: vec![Object::String(Rc::new("foo".to_string()))],
                functions: vec![Function {
                    instructions: vec![Instruction::Constant(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 5)
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();
        assert_eq!(bytecode, expected);
    }
}

#[test]
fn arrays() {
    let tests = [
        (
            "[]",
            Bytecode {
                constants: vec![],
                functions: vec![Function {
                    instructions: vec![Instruction::Array(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 2),
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "[1]",
            Bytecode {
                constants: vec![Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Array(1),
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 2),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "[1, \"foo\"]",
            Bytecode {
                constants: vec![
                    Object::Integer(1),
                    Object::String(Rc::new("foo".to_string())),
                ],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Array(2),
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 2),
                        },
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 9),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 10),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 10),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();
        assert_eq!(bytecode, expected);
    }

    let tests = [(
        "[1, [2, 3], 4]",
        Bytecode {
            constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
            ],
            functions: vec![Function {
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Constant(2),
                    Instruction::Array(2),
                    Instruction::Constant(3),
                    Instruction::Array(3),
                    Instruction::Pop,
                ],
                ranges: vec![],
                nr_local_variables: 0,
                nr_arguments: 0,
            }],
            main_function: 0,
        },
    )];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program).unwrap();
        bytecode.functions[0].ranges = vec![];

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn hash_map() {
    let tests = [
        (
            "{}",
            Bytecode {
                constants: vec![],
                functions: vec![Function {
                    instructions: vec![Instruction::HashMap(0), Instruction::Pop],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 2),
                        };
                        2
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "{1: 2}",
            Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::HashMap(2),
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 2),
                        },
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();
        assert_eq!(bytecode, expected);
    }

    let tests = [(
        "{1: {2: \"foo\"}, \"bar\": 4}",
        Bytecode {
            constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::String(Rc::new("foo".to_string())),
                Object::String(Rc::new("bar".to_string())),
                Object::Integer(4),
            ],
            functions: vec![Function {
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Constant(2),
                    Instruction::HashMap(2),
                    Instruction::Constant(3),
                    Instruction::Constant(4),
                    Instruction::HashMap(4),
                    Instruction::Pop,
                ],
                ranges: vec![],
                nr_local_variables: 0,
                nr_arguments: 0,
            }],
            main_function: 0,
        },
    )];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program).unwrap();
        bytecode.functions[0].ranges = vec![];

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn prefix_operator() {
    let tests = [
        (
            "-10",
            Bytecode {
                constants: vec![Object::Integer(10)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Minus,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "-4.2",
            Bytecode {
                constants: vec![Object::Float(4.2)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Minus,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 4),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 4),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 4),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "!10",
            Bytecode {
                constants: vec![Object::Integer(10)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Bang,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 3),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "!false",
            Bytecode {
                constants: vec![Object::Boolean(false)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Bang,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn while_loop() {
    let tests = [
        (
            "while (true) {}",
            Bytecode {
                constants: vec![Object::Boolean(true)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(3),
                        Instruction::Jump(0),
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 13),
                            end: Position::new(0, 15),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "while (true) {1}",
            Bytecode {
                constants: vec![Object::Boolean(true), Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(5),
                        Instruction::Constant(1),
                        Instruction::Pop,
                        Instruction::Jump(0),
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 14),
                            end: Position::new(0, 15),
                        },
                        Range {
                            start: Position::new(0, 14),
                            end: Position::new(0, 15),
                        },
                        Range {
                            start: Position::new(0, 13),
                            end: Position::new(0, 16),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn assign() {
    let tests = [
        (
            "foo = -1",
            Bytecode {
                constants: vec![Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Minus,
                        Instruction::StoreGlobal(0),
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 6),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 8),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "[foo] = [1]",
            Bytecode {
                constants: vec![Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Array(1),
                        Instruction::UnpackArray(1),
                        Instruction::StoreGlobal(0),
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 9),
                            end: Position::new(0, 10),
                        },
                        Range {
                            start: Position::new(0, 8),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 11),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 11),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "[foo, bar] = [1, 2]",
            Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Array(2),
                        Instruction::UnpackArray(2),
                        Instruction::StoreGlobal(0),
                        Instruction::StoreGlobal(1),
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 14),
                            end: Position::new(0, 15),
                        },
                        Range {
                            start: Position::new(0, 17),
                            end: Position::new(0, 18),
                        },
                        Range {
                            start: Position::new(0, 13),
                            end: Position::new(0, 19),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 19),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 19),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 19),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "foo = []\nfoo[0] = 1",
            Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(0)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Array(0),
                        Instruction::StoreGlobal(0),
                        Instruction::Constant(0),
                        Instruction::LoadGlobal(0),
                        Instruction::Constant(1),
                        Instruction::IndexSet,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 6),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(1, 9),
                            end: Position::new(1, 10),
                        },
                        Range {
                            start: Position::new(1, 0),
                            end: Position::new(1, 3),
                        },
                        Range {
                            start: Position::new(1, 4),
                            end: Position::new(1, 5),
                        },
                        Range {
                            start: Position::new(1, 0),
                            end: Position::new(1, 10),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }

    // Without positions
    let tests = [
        (
            "[foo, [bar, baz]] = [1, [2, 3]]",
            Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Constant(2),
                        Instruction::Array(2),
                        Instruction::Array(2),
                        Instruction::UnpackArray(2),
                        Instruction::StoreGlobal(0),
                        Instruction::UnpackArray(2),
                        Instruction::StoreGlobal(1),
                        Instruction::StoreGlobal(2),
                    ],
                    ranges: vec![],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "foo = {}\n[foo.bar, foo.baz] = [10, 20]",
            Bytecode {
                constants: vec![
                    Object::Integer(10),
                    Object::Integer(20),
                    Object::String(Rc::new("bar".to_string())),
                    Object::String(Rc::new("baz".to_string())),
                ],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::HashMap(0),
                        Instruction::StoreGlobal(0),
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Array(2),
                        Instruction::UnpackArray(2),
                        Instruction::LoadGlobal(0),
                        Instruction::Constant(2),
                        Instruction::IndexSet,
                        Instruction::LoadGlobal(0),
                        Instruction::Constant(3),
                        Instruction::IndexSet,
                    ],
                    ranges: vec![],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program).unwrap();
        bytecode.functions[0].ranges = vec![];

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn infix_operator() {
    let tests = [
        (
            "1 < 2",
            Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Le,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 5),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 5),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "1 > 2",
            Bytecode {
                constants: vec![Object::Integer(2), Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Le,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 5),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 5),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "1 >= 2",
            Bytecode {
                constants: vec![Object::Integer(2), Object::Integer(1)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Leq,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 5),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn index() {
    let tests = [
        (
            "[10][0]",
            Bytecode {
                constants: vec![Object::Integer(10), Object::Integer(0)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::Array(1),
                        Instruction::Constant(1),
                        Instruction::IndexGet,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 3),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 4),
                        },
                        Range {
                            start: Position::new(0, 5),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 7),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 7),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "{}.foo",
            Bytecode {
                constants: vec![Object::String(Rc::new("foo".to_string()))],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::HashMap(0),
                        Instruction::Constant(0),
                        Instruction::IndexGet,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 2),
                        },
                        Range {
                            start: Position::new(0, 3),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 6),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn for_loop() {
    let input = "for (i = 0; i < 10; i = i + 1) {}";

    let expected = Bytecode {
        constants: vec![Object::Integer(0), Object::Integer(10), Object::Integer(1)],
        functions: vec![Function {
            instructions: vec![
                Instruction::Constant(0),
                Instruction::StoreGlobal(0),
                Instruction::LoadGlobal(0),
                Instruction::Constant(1),
                Instruction::Le,
                Instruction::JumpNotTruthy(11),
                Instruction::LoadGlobal(0),
                Instruction::Constant(2),
                Instruction::Add,
                Instruction::StoreGlobal(0),
                Instruction::Jump(2),
            ],
            ranges: vec![
                Range {
                    start: Position::new(0, 9),
                    end: Position::new(0, 10),
                },
                Range {
                    start: Position::new(0, 5),
                    end: Position::new(0, 10),
                },
                Range {
                    start: Position::new(0, 12),
                    end: Position::new(0, 13),
                },
                Range {
                    start: Position::new(0, 16),
                    end: Position::new(0, 18),
                },
                Range {
                    start: Position::new(0, 12),
                    end: Position::new(0, 18),
                },
                Range {
                    start: Position::new(0, 12),
                    end: Position::new(0, 18),
                },
                Range {
                    start: Position::new(0, 24),
                    end: Position::new(0, 25),
                },
                Range {
                    start: Position::new(0, 28),
                    end: Position::new(0, 29),
                },
                Range {
                    start: Position::new(0, 24),
                    end: Position::new(0, 29),
                },
                Range {
                    start: Position::new(0, 20),
                    end: Position::new(0, 29),
                },
                Range {
                    start: Position::new(0, 31),
                    end: Position::new(0, 33),
                },
            ],
            nr_local_variables: 0,
            nr_arguments: 0,
        }],
        main_function: 0,
    };

    let program = parse(input).unwrap();
    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program).unwrap();

    assert_eq!(bytecode, expected);
}

#[test]
fn if_statement() {
    let tests = [
        (
            "if (true) {}",
            Bytecode {
                constants: vec![Object::Boolean(true)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(4),
                        Instruction::Null,
                        Instruction::Jump(5),
                        Instruction::Null,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 10),
                            end: Position::new(0, 12),
                        },
                        Range {
                            start: Position::new(0, 10),
                            end: Position::new(0, 12),
                        },
                        Range {
                            start: Position::new(0, 10),
                            end: Position::new(0, 12),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 12),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "if (true) {} else {}",
            Bytecode {
                constants: vec![Object::Boolean(true)],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(4),
                        Instruction::Null,
                        Instruction::Jump(5),
                        Instruction::Null,
                        Instruction::Pop,
                    ],
                    ranges: vec![
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 8),
                        },
                        Range {
                            start: Position::new(0, 10),
                            end: Position::new(0, 12),
                        },
                        Range {
                            start: Position::new(0, 10),
                            end: Position::new(0, 12),
                        },
                        Range {
                            start: Position::new(0, 18),
                            end: Position::new(0, 20),
                        },
                        Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 20),
                        },
                    ],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();
        assert_eq!(bytecode, expected);
    }

    // Ignore ranges
    let tests = [
        (
            "if (true) {a = 0} else {10}",
            Bytecode {
                constants: vec![
                    Object::Boolean(true),
                    Object::Integer(0),
                    Object::Integer(10),
                ],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(6),
                        Instruction::Constant(1),
                        Instruction::StoreGlobal(0),
                        Instruction::Null,
                        Instruction::Jump(7),
                        Instruction::Constant(2),
                        Instruction::Pop,
                    ],
                    ranges: vec![],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
        (
            "if (true) {a = 0} else if (false) {10}",
            Bytecode {
                constants: vec![
                    Object::Boolean(true),
                    Object::Integer(0),
                    Object::Boolean(false),
                    Object::Integer(10),
                ],
                functions: vec![Function {
                    instructions: vec![
                        Instruction::Constant(0),
                        Instruction::JumpNotTruthy(6),
                        Instruction::Constant(1),
                        Instruction::StoreGlobal(0),
                        Instruction::Null,
                        Instruction::Jump(11),
                        Instruction::Constant(2),
                        Instruction::JumpNotTruthy(10),
                        Instruction::Constant(3),
                        Instruction::Jump(11),
                        Instruction::Null,
                        Instruction::Pop,
                    ],
                    ranges: vec![],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                }],
                main_function: 0,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program).unwrap();
        bytecode.functions[0].ranges = vec![];

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn function_literal() {
    let tests = [
        (
            "fn(){}",
            Bytecode {
                constants: vec![],
                functions: vec![
                    Function {
                        instructions: vec![Instruction::Null, Instruction::Return],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 4),
                                end: Position::new(0, 6),
                            },
                            Range {
                                start: Position::new(0, 4),
                                end: Position::new(0, 6),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                    Function {
                        instructions: vec![
                            Instruction::CreateClosure(CreateClosure {
                                function_index: 0,
                                nr_free_variables: 0,
                            }),
                            Instruction::Pop,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 6),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 6),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                ],
                main_function: 1,
            },
        ),
        (
            "fn(){a = 10}",
            Bytecode {
                constants: vec![Object::Integer(10)],
                functions: vec![
                    Function {
                        instructions: vec![
                            Instruction::Constant(0),
                            Instruction::StoreLocal(0),
                            Instruction::Null,
                            Instruction::Return,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 9),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 4),
                                end: Position::new(0, 12),
                            },
                        ],
                        nr_local_variables: 1,
                        nr_arguments: 0,
                    },
                    Function {
                        instructions: vec![
                            Instruction::CreateClosure(CreateClosure {
                                function_index: 0,
                                nr_free_variables: 0,
                            }),
                            Instruction::Pop,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 12),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 12),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                ],
                main_function: 1,
            },
        ),
        (
            "fn(a){a * 2}",
            Bytecode {
                constants: vec![Object::Integer(2)],
                functions: vec![
                    Function {
                        instructions: vec![
                            Instruction::LoadLocal(0),
                            Instruction::Constant(0),
                            Instruction::Multiply,
                            Instruction::Return,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 6),
                                end: Position::new(0, 7),
                            },
                            Range {
                                start: Position::new(0, 10),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 6),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 12),
                            },
                        ],
                        nr_local_variables: 1,
                        nr_arguments: 1,
                    },
                    Function {
                        instructions: vec![
                            Instruction::CreateClosure(CreateClosure {
                                function_index: 0,
                                nr_free_variables: 0,
                            }),
                            Instruction::Pop,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 12),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 12),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                ],
                main_function: 1,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }

    // No ranges
    let tests = [(
        r#"
        fn(a) {
            fn(b) {
                fn(c) {
                    a + b + c
                }
            }
        }

        "#,
        Bytecode {
            constants: vec![],
            functions: vec![
                Function {
                    instructions: vec![
                        Instruction::LoadFree(0),
                        Instruction::LoadFree(1),
                        Instruction::Add,
                        Instruction::LoadLocal(0),
                        Instruction::Add,
                        Instruction::Return,
                    ],
                    ranges: vec![],
                    nr_local_variables: 1,
                    nr_arguments: 1,
                },
                Function {
                    instructions: vec![
                        Instruction::LoadFree(0),
                        Instruction::LoadLocal(0),
                        Instruction::CreateClosure(CreateClosure {
                            function_index: 0,
                            nr_free_variables: 2,
                        }),
                        Instruction::Return,
                    ],
                    ranges: vec![],
                    nr_local_variables: 1,
                    nr_arguments: 1,
                },
                Function {
                    instructions: vec![
                        Instruction::LoadLocal(0),
                        Instruction::CreateClosure(CreateClosure {
                            function_index: 1,
                            nr_free_variables: 1,
                        }),
                        Instruction::Return,
                    ],
                    ranges: vec![],
                    nr_local_variables: 1,
                    nr_arguments: 1,
                },
                Function {
                    instructions: vec![
                        Instruction::CreateClosure(CreateClosure {
                            function_index: 2,
                            nr_free_variables: 0,
                        }),
                        Instruction::Pop,
                    ],
                    ranges: vec![],
                    nr_local_variables: 0,
                    nr_arguments: 0,
                },
            ],
            main_function: 3,
        },
    )];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program).unwrap();
        for fun in bytecode.functions.iter_mut() {
            fun.ranges = vec![];
        }

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn fn_call() {
    let tests = [
        (
            "fn(){1}()",
            Bytecode {
                constants: vec![Object::Integer(1)],
                functions: vec![
                    Function {
                        instructions: vec![Instruction::Constant(0), Instruction::Return],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 6),
                            },
                            Range {
                                start: Position::new(0, 4),
                                end: Position::new(0, 7),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                    Function {
                        instructions: vec![
                            Instruction::CreateClosure(CreateClosure {
                                function_index: 0,
                                nr_free_variables: 0,
                            }),
                            Instruction::FnCall(0),
                            Instruction::Pop,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 7),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 9),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 9),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                ],
                main_function: 1,
            },
        ),
        (
            "fn(a){1}(5)",
            Bytecode {
                constants: vec![Object::Integer(5), Object::Integer(1)],
                functions: vec![
                    Function {
                        instructions: vec![Instruction::Constant(1), Instruction::Return],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 6),
                                end: Position::new(0, 7),
                            },
                            Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 8),
                            },
                        ],
                        nr_local_variables: 1,
                        nr_arguments: 1,
                    },
                    Function {
                        instructions: vec![
                            Instruction::Constant(0),
                            Instruction::CreateClosure(CreateClosure {
                                function_index: 0,
                                nr_free_variables: 0,
                            }),
                            Instruction::FnCall(1),
                            Instruction::Pop,
                        ],
                        ranges: vec![
                            Range {
                                start: Position::new(0, 9),
                                end: Position::new(0, 10),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 8),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 11),
                            },
                            Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 11),
                            },
                        ],
                        nr_local_variables: 0,
                        nr_arguments: 0,
                    },
                ],
                main_function: 1,
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();

        assert_eq!(bytecode, expected);
    }
}

#[test]
fn builtin() {
    let input = r#"len("foo")"#;

    let program = parse(input).unwrap();
    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program).unwrap();

    assert_eq!(
        bytecode,
        Bytecode {
            constants: vec![Object::String(Rc::new("foo".to_string()))],
            functions: vec![Function {
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Builtin(Builtin::Len),
                    Instruction::FnCall(1),
                    Instruction::Pop,
                ],
                ranges: vec![
                    Range {
                        start: Position::new(0, 4),
                        end: Position::new(0, 9)
                    },
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 3)
                    },
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 10)
                    },
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 10)
                    }
                ],
                nr_local_variables: 0,
                nr_arguments: 0
            }],
            main_function: 0
        }
    );
}
