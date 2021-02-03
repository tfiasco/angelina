use crate::parser::ast::Expr;

#[derive(Debug, Clone)]
pub enum Operator {
    VertexFullScan {
        element_name: String,
    },
    VertexIdRangeScan {
        element_name: String,
        range: (Option<Expr>, Option<Expr>),
    },
    VertexLookup {
        element_name: String,
        vertex_id: Expr,
    },
    OutEdgeSeqScan {
        element_name: String,
        edge_label: Option<Expr>,
        src: Option<Expr>,
    },
    InEdgeSeqScan {
        element_name: String,
        edge_label: Option<Expr>,
        dst: Option<Expr>,
    },
    OutEdgeLookup {
        element_name: String,
        edge_label: Expr,
        src: Expr,
        dst: Expr,
    },
    InEdgeLookup {
        element_name: String,
        edge_label: Expr,
        src: Expr,
        dst: Expr,
    },
    PredicateFilter {
        source: Box<Operator>,
        predicates: Vec<Expr>,
    },
    Projection {
        source: Box<Operator>,
        items: Vec<Expr>,
    },
    SimplePathJoin {
        operators: Vec<Operator>,
    },
}
