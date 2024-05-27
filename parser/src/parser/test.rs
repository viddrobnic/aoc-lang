use crate::{
    ast,
    error::{Error, ErrorKind, Result},
    position::{Position, Range},
};

use super::parse;

#[test]
fn empty_program() -> Result<()> {
    let program = parse("\n\n   \n")?;
    assert_eq!(program.statements, vec![]);

    Ok(())
}

#[test]
fn simple_prefix_expression() -> Result<()> {
    let input = r#"
        foo
        10
        4.2
        true
        false
        "bar"
        break
        continue
        // this is a comment
        foo // inline comment
    "#;

    let program = parse(input)?;
    assert_eq!(
        program.statements,
        vec![
            ast::Node {
                value: ast::NodeValue::Identifier("foo".to_string()),
                range: Range {
                    start: Position::new(1, 8),
                    end: Position::new(1, 11)
                },
            },
            ast::Node {
                value: ast::NodeValue::IntegerLiteral(10),
                range: Range {
                    start: Position::new(2, 8),
                    end: Position::new(2, 10)
                },
            },
            ast::Node {
                value: ast::NodeValue::FloatLiteral(4.2),
                range: Range {
                    start: Position::new(3, 8),
                    end: Position::new(3, 11)
                },
            },
            ast::Node {
                value: ast::NodeValue::BoolLiteral(true),
                range: Range {
                    start: Position::new(4, 8),
                    end: Position::new(4, 12)
                },
            },
            ast::Node {
                value: ast::NodeValue::BoolLiteral(false),
                range: Range {
                    start: Position::new(5, 8),
                    end: Position::new(5, 13)
                },
            },
            ast::Node {
                value: ast::NodeValue::StringLiteral("bar".to_string()),
                range: Range {
                    start: Position::new(6, 8),
                    end: Position::new(6, 13)
                },
            },
            ast::Node {
                value: ast::NodeValue::Break,
                range: Range {
                    start: Position::new(7, 8),
                    end: Position::new(7, 13)
                },
            },
            ast::Node {
                value: ast::NodeValue::Continue,
                range: Range {
                    start: Position::new(8, 8),
                    end: Position::new(8, 16)
                },
            },
            ast::Node {
                value: ast::NodeValue::Comment("this is a comment".to_string()),
                range: Range {
                    start: Position::new(9, 8),
                    end: Position::new(9, 28)
                },
            },
            ast::Node {
                value: ast::NodeValue::Identifier("foo".to_string()),
                range: Range {
                    start: Position::new(10, 8),
                    end: Position::new(10, 11)
                },
            },
            ast::Node {
                value: ast::NodeValue::Comment("inline comment".to_string()),
                range: Range {
                    start: Position::new(10, 12),
                    end: Position::new(10, 29)
                },
            },
        ]
    );

    Ok(())
}

#[test]
fn one_node_per_line() {
    let program = parse("foo bar");
    assert_eq!(
        program,
        Err(Error {
            kind: ErrorKind::ExpectedEol,
            range: Range {
                start: Position::new(0, 4),
                end: Position::new(0, 7),
            }
        })
    )
}

#[test]
fn prefix_operator() -> Result<()> {
    let tests = [
        (
            "!false",
            ast::Node {
                value: ast::NodeValue::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::BoolLiteral(false),
                        range: Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 6),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 6),
                },
            },
        ),
        (
            "-42",
            ast::Node {
                value: ast::NodeValue::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Negative,
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(42),
                        range: Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 3),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 3),
                },
            },
        ),
        (
            "--1",
            ast::Node {
                value: ast::NodeValue::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Negative,
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::PrefixOperator {
                            operator: ast::PrefixOperatorKind::Negative,
                            right: Box::new(ast::Node {
                                value: ast::NodeValue::IntegerLiteral(1),
                                range: Range {
                                    start: Position::new(0, 2),
                                    end: Position::new(0, 3),
                                },
                            }),
                        },
                        range: Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 3),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 3),
                },
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.statements[0], expected);
    }

    Ok(())
}

#[test]
fn infix_opeartor() -> Result<()> {
    let tests = [
        (
            "1+2",
            ast::Node {
                value: ast::NodeValue::InfixOperator {
                    operator: ast::InfixOperatorKind::Add,
                    left: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                    }),
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(2),
                        range: Range {
                            start: Position::new(0, 2),
                            end: Position::new(0, 3),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 3),
                },
            },
        ),
        (
            "1 & 2",
            ast::Node {
                value: ast::NodeValue::InfixOperator {
                    operator: ast::InfixOperatorKind::And,
                    left: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                    }),
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(2),
                        range: Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 5),
                },
            },
        ),
        (
            "1 & 2 + 3",
            ast::Node {
                value: ast::NodeValue::InfixOperator {
                    operator: ast::InfixOperatorKind::And,
                    left: Box::new(ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                    }),
                    right: Box::new(ast::Node {
                        value: ast::NodeValue::InfixOperator {
                            operator: ast::InfixOperatorKind::Add,
                            left: Box::new(ast::Node {
                                value: ast::NodeValue::IntegerLiteral(2),
                                range: Range {
                                    start: Position::new(0, 4),
                                    end: Position::new(0, 5),
                                },
                            }),
                            right: Box::new(ast::Node {
                                value: ast::NodeValue::IntegerLiteral(3),
                                range: Range {
                                    start: Position::new(0, 8),
                                    end: Position::new(0, 9),
                                },
                            }),
                        },
                        range: Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 9),
                        },
                    }),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 9),
                },
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.statements[0], expected);
    }

    Ok(())
}

#[test]
fn grouped() -> Result<()> {
    let program = parse("1 + ((2 + (3 == 4)) + 2)")?;

    assert_eq!(program.statements.len(), 1);
    assert_eq!(
        program.statements[0],
        ast::Node {
            value: ast::NodeValue::InfixOperator {
                operator: ast::InfixOperatorKind::Add,
                left: Box::new(ast::Node {
                    value: ast::NodeValue::IntegerLiteral(1),
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1)
                    }
                }),
                right: Box::new(ast::Node {
                    value: ast::NodeValue::InfixOperator {
                        operator: ast::InfixOperatorKind::Add,
                        left: Box::new(ast::Node {
                            value: ast::NodeValue::InfixOperator {
                                operator: ast::InfixOperatorKind::Add,
                                left: Box::new(ast::Node {
                                    value: ast::NodeValue::IntegerLiteral(2),
                                    range: Range {
                                        start: Position::new(0, 6),
                                        end: Position::new(0, 7),
                                    }
                                }),
                                right: Box::new(ast::Node {
                                    value: ast::NodeValue::InfixOperator {
                                        operator: ast::InfixOperatorKind::Eq,
                                        left: Box::new(ast::Node {
                                            value: ast::NodeValue::IntegerLiteral(3),
                                            range: Range {
                                                start: Position::new(0, 11),
                                                end: Position::new(0, 12),
                                            }
                                        }),
                                        right: Box::new(ast::Node {
                                            value: ast::NodeValue::IntegerLiteral(4),
                                            range: Range {
                                                start: Position::new(0, 16),
                                                end: Position::new(0, 17),
                                            }
                                        })
                                    },
                                    range: Range {
                                        start: Position::new(0, 10),
                                        end: Position::new(0, 18),
                                    }
                                })
                            },
                            range: Range {
                                start: Position::new(0, 5),
                                end: Position::new(0, 19),
                            }
                        }),
                        right: Box::new(ast::Node {
                            value: ast::NodeValue::IntegerLiteral(2),
                            range: Range {
                                start: Position::new(0, 22),
                                end: Position::new(0, 23)
                            }
                        })
                    },
                    range: Range {
                        start: Position::new(0, 4),
                        end: Position::new(0, 24),
                    }
                })
            },
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 24),
            }
        }
    );

    Ok(())
}

#[test]
fn array_literal() -> Result<()> {
    let tests = [
        (
            "[]",
            ast::Node {
                value: ast::NodeValue::ArrayLiteral(vec![]),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 2),
                },
            },
        ),
        (
            "[1]",
            ast::Node {
                value: ast::NodeValue::ArrayLiteral(vec![ast::Node {
                    value: ast::NodeValue::IntegerLiteral(1),
                    range: Range {
                        start: Position::new(0, 1),
                        end: Position::new(0, 2),
                    },
                }]),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 3),
                },
            },
        ),
        (
            "[1, 2]",
            ast::Node {
                value: ast::NodeValue::ArrayLiteral(vec![
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 2),
                        },
                    },
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(2),
                        range: Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                    },
                ]),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 6),
                },
            },
        ),
        (
            "[1, 2,]",
            ast::Node {
                value: ast::NodeValue::ArrayLiteral(vec![
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(0, 1),
                            end: Position::new(0, 2),
                        },
                    },
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(2),
                        range: Range {
                            start: Position::new(0, 4),
                            end: Position::new(0, 5),
                        },
                    },
                ]),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 7),
                },
            },
        ),
        (
            r#"[
                    1,
                    2,
                ]"#,
            ast::Node {
                value: ast::NodeValue::ArrayLiteral(vec![
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(1),
                        range: Range {
                            start: Position::new(1, 20),
                            end: Position::new(1, 21),
                        },
                    },
                    ast::Node {
                        value: ast::NodeValue::IntegerLiteral(2),
                        range: Range {
                            start: Position::new(2, 20),
                            end: Position::new(2, 21),
                        },
                    },
                ]),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(3, 17),
                },
            },
        ),
    ];

    for (input, expected) in tests {
        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.statements[0], expected);
    }

    Ok(())
}

#[test]
fn precedence() -> Result<()> {
    let tests = [
        ("1 + 2 + 3", "((1 + 2) + 3)"),
        ("1 + 2 * 3", "(1 + (2 * 3))"),
        ("1 + 2 == 3", "((1 + 2) == 3)"),
        ("1 != 2 & false", "((1 != 2) & false)"),
        ("a & b | c", "((a & b) | c)"),
        ("a | b & c", "(a | (b & c))"),
        ("2 <= 3 == 3 > 2", "((2 <= 3) == (3 > 2))"),
        ("-1 + 1 * 2 % 3 / 4", "((-1) + (((1 * 2) % 3) / 4))"),
        ("1 + -2", "(1 + (-2))"),
        ("1 * (2 + 3)", "(1 * (2 + 3))"),
        ("[1 + 2, 3 + 4 * 5]", "[(1 + 2), (3 + (4 * 5))]"),
    ];

    for (input, expected) in tests {
        let program = parse(input)?;
        assert_eq!(program.to_string(), expected);
    }

    Ok(())
}
