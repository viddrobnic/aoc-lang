use crate::{
    ast::{HashLiteralPair, Node, NodeKind, NodeValue},
    error::{Error, ErrorKind, Result},
};

pub fn validate_hash_literal(items: &[HashLiteralPair]) -> Result<()> {
    for it in items {
        if it.key.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: it.key.kind(),
                },
                range: it.key.range,
            });
        }

        if it.value.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: it.value.kind(),
                },
                range: it.value.range,
            });
        }
    }

    Ok(())
}

pub fn validate_array_literal(items: &[Node]) -> Result<()> {
    for it in items {
        if it.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Statement,
                    got: it.kind(),
                },
                range: it.range,
            });
        }
    }

    Ok(())
}

pub fn validate_assignee(assignee: &Node) -> Result<()> {
    match &assignee.value {
        NodeValue::Identifier(_) => (),
        NodeValue::Index { .. } => (),
        NodeValue::ArrayLiteral(arr) => {
            for it in arr {
                validate_assignee(it)?;
            }
        }
        _ => {
            return Err(Error {
                kind: ErrorKind::InvalidAssignee,
                range: assignee.range,
            })
        }
    }

    Ok(())
}