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
