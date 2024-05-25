use crate::{
    ast,
    error::{Error, ErrorKind, Result},
    position::Position,
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
                position: Position::new(1, 8)
            },
            ast::Node {
                value: ast::NodeValue::IntegerLiteral(10),
                position: Position::new(2, 8)
            },
            ast::Node {
                value: ast::NodeValue::FloatLiteral(4.2),
                position: Position::new(3, 8)
            },
            ast::Node {
                value: ast::NodeValue::BoolLiteral(true),
                position: Position::new(4, 8)
            },
            ast::Node {
                value: ast::NodeValue::BoolLiteral(false),
                position: Position::new(5, 8)
            },
            ast::Node {
                value: ast::NodeValue::StringLiteral("bar".to_string()),
                position: Position::new(6, 8)
            },
            ast::Node {
                value: ast::NodeValue::Break,
                position: Position::new(7, 8)
            },
            ast::Node {
                value: ast::NodeValue::Continue,
                position: Position::new(8, 8)
            },
            ast::Node {
                value: ast::NodeValue::Comment("this is a comment".to_string()),
                position: Position::new(9, 8)
            },
            ast::Node {
                value: ast::NodeValue::Identifier("foo".to_string()),
                position: Position::new(10, 8)
            },
            ast::Node {
                value: ast::NodeValue::Comment("inline comment".to_string()),
                position: Position::new(10, 12)
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
            position: Position::new(0, 4),
        })
    )
}
