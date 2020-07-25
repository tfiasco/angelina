use std::cell::RefCell;
use std::rc::Rc;

use crate::parser::operator::{BinaryOperator, UnaryOperator};

#[derive(Debug)]
pub enum Statement {
    /// SHOW SCHEMA
    ShowVertexLabels,
    ShowEdgeLabels,
    ShowPropertyKeys,
    /// CREATE SCHEMA
    CreateGraph {
        name: String,
    },
    CreateVertexLabel {
        name: String,
    },
    CreateEdgeLabel {
        name: String,
        multiplicity: String,
    },
    CreatePropertyKey {
        name: String,
        cardinality: String,
    },
    /// DROP SCHEMA
    DropGraph {
        name: String,
    },
    DropVertexLabel {
        name: String,
    },
    DropEdgeLabel {
        name: String,
    },
    DropPropertyKey {
        name: String,
    },
    /// INSERT
    InsertVertex {
        label: Expr,
        properties: Vec<String>,
        vertex_id: Expr,
        values: Vec<Expr>,
    },
    InsertEdge {
        label: Expr,
        properties: Vec<String>,
        src_vertex_id: Expr,
        dst_vertex_id: Expr,
        values: Vec<Expr>,
    },
    /// UPDATE
    Update {
        operation: Vec<PropertyUpdateOp>,
    },
    /// DELETE
    Delete {
        elements: Vec<Expr>,
    },
    /// Select
    Select {
        items: Vec<Expr>,
        from: GraphPattern,
        condition: Option<Expr>,
    },
}

#[derive(Debug)]
pub enum PropertyUpdateOp {
    Update { property: Expr, value: Expr },
    Delete { property: Expr },
}

#[derive(Debug)]
pub struct GraphPattern {
    pub(crate) triplets: Vec<GraphTriplet>,
}

#[derive(Debug)]
pub struct GraphTriplet {
    pub(crate) src: Box<Expr>,
    pub(crate) edge: Box<Expr>,
    pub(crate) dst: Box<Expr>,
}

#[derive(Debug)]
pub struct GraphPath {
    head: VertexExpr,
}

#[derive(Debug)]
pub struct VertexExpr {
    pub(crate) value: Expr,
    pub(crate) out_edge: Option<Rc<RefCell<EdgeExpr>>>,
    pub(crate) in_edge: Option<Rc<RefCell<EdgeExpr>>>,
}

#[derive(Debug)]
pub struct EdgeExpr {
    pub(crate) value: Expr,
    pub(crate) src_vertex: Option<Rc<RefCell<VertexExpr>>>,
    pub(crate) dst_vertex: Option<Rc<RefCell<VertexExpr>>>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    /// literals
    Value(Value),
    /// identifiers: vertex_1, edge_1, prop_1 ...
    Identifier(String),
    /// vertex_1.prop_1, edge_1.prop_2, vertex_2.prop_3.source ...
    CompoundIdentifier(Vec<String>),
    /// *
    Wildcard,
    /// vertex_1.*
    CompoundWildcard(Vec<String>),
    /// func(a, b, c)
    Function {
        func_name: String,
        arguments: Vec<Expr>,
    },
    /// -1, NOT NULL ...
    UnaryOp { op: UnaryOperator, expr: Box<Expr> },
    /// 1 + 2, a > 0 ...
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// (a + b), (a AND b)
    Nested(Box<Expr>),
    /// a.label
    LabelExpr(Box<Expr>),
    /// a.id
    IdExpr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(String),
    String(String),
    Boolean(bool),
    Null,
}
