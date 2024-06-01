use std::rc::Rc;

use parser::{
    parse,
    position::{Position, Range},
};

use crate::{
    bytecode::{Bytecode, Instruction},
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
                instructions: vec![Instruction::Constant(0), Instruction::Pop],
                ranges: vec![
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 3)
                    };
                    2
                ],
            },
        ),
        (
            "4.2",
            Bytecode {
                constants: vec![Object::Float(4.2)],
                instructions: vec![Instruction::Constant(0), Instruction::Pop],
                ranges: vec![
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 3)
                    };
                    2
                ],
            },
        ),
        (
            "true",
            Bytecode {
                constants: vec![Object::Boolean(true)],
                instructions: vec![Instruction::Constant(0), Instruction::Pop],
                ranges: vec![
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 4)
                    };
                    2
                ],
            },
        ),
        (
            "\"foo\"",
            Bytecode {
                constants: vec![Object::String(Rc::new("foo".to_string()))],
                instructions: vec![Instruction::Constant(0), Instruction::Pop],
                ranges: vec![
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 5)
                    };
                    2
                ],
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program);
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
                instructions: vec![Instruction::Array(0), Instruction::Pop],
                ranges: vec![
                    Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 2),
                    };
                    2
                ],
            },
        ),
        (
            "[1]",
            Bytecode {
                constants: vec![Object::Integer(1)],
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
            },
        ),
        (
            "[1, \"foo\"]",
            Bytecode {
                constants: vec![
                    Object::Integer(1),
                    Object::String(Rc::new("foo".to_string())),
                ],
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
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let bytecode = compiler.compile(&program);
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
        },
    )];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let compiler = Compiler::new();
        let mut bytecode = compiler.compile(&program);
        bytecode.ranges = vec![];

        assert_eq!(bytecode, expected);
    }
}
