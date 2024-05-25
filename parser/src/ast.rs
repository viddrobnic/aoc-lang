use crate::position::Position;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    value: NodeValue,
    position: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeValue {
    Identifier(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    ArrayLiteral(Vec<Node>),
    HashLiteral(Vec<HashLiteralPair>),
    PrefixOperator {
        operator: PrefixOperatorKind,
        right: Box<Node>,
    },
    BinaryOperator {
        operator: InfixOperatorKind,
        left: Box<Node>,
        right: Box<Node>,
    },
    Assign {
        ident: Box<Node>,
        value: Box<Node>,
    },
    Index {
        left: Box<Node>,
        index: Box<Node>,
    },
    If {
        condition: Box<Node>,
        consequence: Vec<Node>,
        alternative: Vec<Node>,
    },
    While {
        condition: Box<Node>,
        body: Vec<Node>,
    },
    For {
        initial: Box<Node>,
        condition: Box<Node>,
        after: Box<Node>,
        body: Vec<Node>,
    },
    Break,
    Continue,
    FunctionLiteral {
        name: Option<String>,
        parameters: Vec<String>,
        body: Vec<Node>,
    },
    FunctionCall {
        function: Box<Node>,
        arguments: Vec<Node>,
    },
    Return(Box<Node>),
    Use(String),
    Comment(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct HashLiteralPair {
    key: Node,
    value: Node,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrefixOperatorKind {
    Not,
    Negative,
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

impl NodeValue {
    pub fn kind(&self) -> NodeKind {
        match self {
            NodeValue::Assign { .. } => NodeKind::Statement,
            NodeValue::While { .. } => NodeKind::Statement,
            NodeValue::For { .. } => NodeKind::Statement,
            NodeValue::Break => NodeKind::Statement,
            NodeValue::Continue => NodeKind::Statement,
            NodeValue::Return(_) => NodeKind::Statement,
            NodeValue::Comment(_) => NodeKind::Statement,
            _ => NodeKind::Expression,
        }
    }
}
