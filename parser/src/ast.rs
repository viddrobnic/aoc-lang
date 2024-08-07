use std::fmt::Display;

use crate::position::Range;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Node>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub value: NodeValue,
    pub range: Range,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeValue {
    Null,
    Identifier(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(u8),
    BoolLiteral(bool),
    StringLiteral(String),
    ArrayLiteral(Vec<Node>),
    HashLiteral(Vec<HashLiteralPair>),
    PrefixOperator(PrefixOperator),
    InfixOperator(InfixOperator),
    Assign(Assign),
    Index(Index),
    If(IfNode),
    While(While),
    For(For),
    Break,
    Continue,
    FunctionLiteral(FunctionLiteral),
    FunctionCall(FunctionCall),
    Return(Box<Node>),
    Use(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Comment {
    pub comment: String,
    pub range: Range,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrefixOperator {
    pub operator: PrefixOperatorKind,
    pub right: Box<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfixOperator {
    pub operator: InfixOperatorKind,
    pub left: Box<Node>,
    pub right: Box<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub ident: Box<Node>,
    pub value: Box<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Index {
    pub left: Box<Node>,
    pub index: Box<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub condition: Box<Node>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub initial: Box<Node>,
    pub condition: Box<Node>,
    pub after: Box<Node>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParamter {
    pub name: String,
    pub range: Range,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionLiteral {
    pub name: Option<String>,
    pub parameters: Vec<FunctionParamter>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub function: Box<Node>,
    pub arguments: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct HashLiteralPair {
    pub key: Node,
    pub value: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfNode {
    pub condition: Box<Node>,
    pub consequence: Block,
    pub alternative: Option<Block>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub nodes: Vec<Node>,
    pub range: Range,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrefixOperatorKind {
    Not,
    Negative,
}

impl Display for PrefixOperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefixOperatorKind::Not => write!(f, "!"),
            PrefixOperatorKind::Negative => write!(f, "-"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InfixOperatorKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Le,
    Leq,
    Ge,
    Geq,
    Eq,
    Neq,
}

impl Display for InfixOperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfixOperatorKind::Add => write!(f, "+"),
            InfixOperatorKind::Subtract => write!(f, "-"),
            InfixOperatorKind::Multiply => write!(f, "*"),
            InfixOperatorKind::Divide => write!(f, "/"),
            InfixOperatorKind::Modulo => write!(f, "%"),
            InfixOperatorKind::And => write!(f, "&"),
            InfixOperatorKind::Or => write!(f, "|"),
            InfixOperatorKind::Le => write!(f, "<"),
            InfixOperatorKind::Leq => write!(f, "<="),
            InfixOperatorKind::Ge => write!(f, ">"),
            InfixOperatorKind::Geq => write!(f, ">="),
            InfixOperatorKind::Eq => write!(f, "=="),
            InfixOperatorKind::Neq => write!(f, "!="),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeKind {
    Expression,
    Statement,
}

impl Node {
    pub fn kind(&self) -> NodeKind {
        self.value.kind()
    }
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Expression => write!(f, "EXPRESSION"),
            NodeKind::Statement => write!(f, "STATEMENT"),
        }
    }
}

impl NodeValue {
    pub fn kind(&self) -> NodeKind {
        match self {
            NodeValue::Assign { .. } => NodeKind::Statement,
            NodeValue::While { .. } => NodeKind::Statement,
            NodeValue::For { .. } => NodeKind::Statement,
            NodeValue::Break => NodeKind::Statement,
            NodeValue::Continue => NodeKind::Statement,
            NodeValue::Return(_) => NodeKind::Statement,
            _ => NodeKind::Expression,
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = self
            .statements
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}", body)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for NodeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeValue::Null => write!(f, "null"),
            NodeValue::Identifier(ident) => write!(f, "{ident}"),
            NodeValue::IntegerLiteral(int) => write!(f, "{int}"),
            NodeValue::FloatLiteral(float) => write!(f, "{float}"),
            NodeValue::CharLiteral(ch) => write!(f, "{}", *ch as char),
            NodeValue::BoolLiteral(boolean) => write!(f, "{boolean}"),
            NodeValue::StringLiteral(string) => write!(f, "\"{string}\""),
            NodeValue::ArrayLiteral(arr) => {
                let elts = arr
                    .iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "[{elts}]")
            }
            NodeValue::HashLiteral(hash) => {
                let elts = hash
                    .iter()
                    .map(|pair| format!("{}: {}", pair.key, pair.value))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "{{{elts}}}")
            }
            NodeValue::PrefixOperator(prefix) => write!(f, "({}{})", prefix.operator, prefix.right),
            NodeValue::InfixOperator(InfixOperator {
                operator,
                left,
                right,
            }) => write!(f, "({left} {operator} {right})"),
            NodeValue::Assign(Assign { ident, value }) => write!(f, "({ident} = {value})"),
            NodeValue::Index(Index { left, index }) => write!(f, "({left}[{index}])"),
            NodeValue::If(if_node) => {
                let cons = if_node
                    .consequence
                    .nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

                let mut alt = String::new();
                if let Some(alternative) = &if_node.alternative {
                    alt = alternative
                        .nodes
                        .iter()
                        .map(|node| node.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                }

                write!(
                    f,
                    "if ({}) {{{}}} else {{{}}}",
                    if_node.condition, cons, alt
                )
            }
            NodeValue::While(while_loop) => {
                let body = while_loop
                    .body
                    .nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

                write!(f, "while ({}) {{{}}}", while_loop.condition, body)
            }
            NodeValue::For(For {
                initial,
                condition,
                after,
                body,
            }) => {
                let body = body
                    .nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

                write!(f, "for ({initial}; {condition}; {after}) {{{body}}}")
            }
            NodeValue::Break => write!(f, "break"),
            NodeValue::Continue => write!(f, "continue"),
            NodeValue::FunctionLiteral(FunctionLiteral {
                name: _,
                parameters,
                body,
            }) => {
                let body = body
                    .nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

                let parameters = parameters
                    .iter()
                    .map(|par| par.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "fn({}) {{{}}}", parameters, body)
            }
            NodeValue::FunctionCall(FunctionCall {
                function,
                arguments,
            }) => {
                let args = arguments
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "({}({}))", function, args)
            }
            NodeValue::Return(ret) => write!(f, "return {ret}"),
            NodeValue::Use(name) => write!(f, "use {name}"),
        }
    }
}
