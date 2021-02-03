use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Deref;

use crate::datamodel::constants::MAX_ID_LENGTH;
use crate::execution::operator::Operator;
use crate::execution::operator::Operator::OutEdgeSeqScan;
use crate::execution::scope::{Comparator, EdgePattern, Scope, VertexPattern};
use crate::parser::ast::Value;
use crate::parser::ast::{Expr, GraphPattern};
use crate::parser::operator::BinaryOperator;
use crate::parser::tokenizer::Token::Keyword;

pub struct Planner {
    scope: Scope,
}

impl Planner {
    pub fn new() -> Self {
        Planner {
            scope: Scope::new(),
        }
    }

    pub fn build_select_query(
        &mut self,
        items: &Vec<Expr>,
        graph_pattern: &GraphPattern,
        condition: &Option<Expr>,
    ) -> Operator {
        self.scope
            .parse_select_query(items, graph_pattern, condition);

        let mut elements = HashSet::new();
        // paths
        let mut path_ops = vec![];
        for (src, edge, dst) in &self.scope.paths.clone() {
            let src_pattern = self.scope.vertices.get(src).unwrap().clone();
            if !elements.contains(&src_pattern.name) {
                path_ops.push(self.build_vertex_pattern(&src_pattern));
                elements.insert(src_pattern.name.to_owned());
            }
            let edge_pattern = self.scope.edges.get(edge).unwrap().clone();
            if !elements.contains(&edge_pattern.name) {
                path_ops.push(self.build_edge_pattern(&edge_pattern));
                elements.insert(edge_pattern.name.to_owned());
            }
            let mut dst_pattern = self.scope.vertices.get(dst).unwrap().clone();
            match &dst_pattern.id[..] {
                [] => {
                    dst_pattern.id = vec![Comparator::Eq(Expr::CompoundIdentifier(vec![
                        edge_pattern.name,
                        "dst".to_string(),
                    ]))]
                }
                _ => dst_pattern.predicates.push(Expr::BinaryOp {
                    op: BinaryOperator::Eq,
                    left: Box::new(Expr::IdExpr(dst_pattern.name.to_owned())),
                    right: Box::new(Expr::CompoundIdentifier(vec![
                        edge_pattern.name,
                        "dst".to_string(),
                    ])),
                }),
            }
            if !elements.contains(&dst_pattern.name) {
                path_ops.push(self.build_vertex_pattern(&dst_pattern));
                elements.insert(dst_pattern.name.to_owned());
            }
        }
        let mut op = match &path_ops[..] {
            [] => panic!("invalid path specification"),
            [op] => op.clone(),
            ops => Operator::SimplePathJoin {
                operators: ops.to_vec(),
            },
        };
        if let Some(expr) = condition {
            op = Operator::PredicateFilter {
                source: Box::new(op),
                predicates: vec![expr.clone()],
            }
        }
        op = Operator::Projection {
            source: Box::new(op),
            items: items.clone(),
        };
        op
    }

    pub fn build_vertex_pattern(&mut self, vertex: &VertexPattern) -> Operator {
        // default FullScan all vertices.
        let mut op;
        let mut predicates = vec![];
        // id range.
        if vertex.id.is_empty() {
            op = Operator::VertexFullScan {
                element_name: vertex.name.to_string(),
            };
        } else if vertex.id.len() == 1 {
            match vertex.id.get(0).unwrap() {
                Comparator::Eq(value) => {
                    op = Operator::VertexLookup {
                        element_name: vertex.name.to_string(),
                        vertex_id: value.clone(),
                    }
                }
                Comparator::Gte(value) => {
                    op = Operator::VertexIdRangeScan {
                        element_name: vertex.name.to_string(),
                        range: (Some(value.clone()), None),
                    }
                }
                Comparator::Lte(value) => {
                    op = Operator::VertexIdRangeScan {
                        element_name: vertex.name.to_string(),
                        range: (None, Some(value.clone())),
                    }
                }
            }
        } else {
            let mut min_values: Vec<Expr> = vec![];
            let mut max_values: Vec<Expr> = vec![];
            for comp in &vertex.id {
                match comp {
                    Comparator::Eq(_) => panic!("invalid Equal operator"),
                    Comparator::Gte(value) => min_values.push(value.clone()),
                    Comparator::Lte(value) => max_values.push(value.clone()),
                }
            }
            let min_value_expr = Expr::Function {
                func_name: "min".to_string(),
                arguments: min_values,
            };
            let max_value_expr = Expr::Function {
                func_name: "max".to_string(),
                arguments: max_values,
            };
            op = Operator::VertexIdRangeScan {
                element_name: vertex.name.to_string(),
                range: (Some(min_value_expr), Some(max_value_expr)),
            }
        }
        // label
        if let Some(label) = &vertex.label {
            predicates.push(Expr::BinaryOp {
                op: BinaryOperator::Eq,
                left: Box::new(Expr::LabelExpr(vertex.name.to_owned())),
                right: Box::new(Expr::Value(Value::String(label.to_owned()))),
            })
        }
        // conditions
        predicates.extend(vertex.predicates.clone());
        if !predicates.is_empty() {
            op = Operator::PredicateFilter {
                source: Box::new(op),
                predicates,
            }
        }
        // projection
        if !vertex.projections.is_empty() {
            op = Operator::Projection {
                source: Box::new(op),
                items: vertex.projections.clone(),
            }
        }
        op
    }

    pub fn build_edge_pattern(&mut self, edge: &EdgePattern) -> Operator {
        let edge_label = match &edge.label {
            Some(label) => Some(Expr::Value(Value::String(label.to_owned()))),
            None => None,
        };
        let src = Some(Expr::Identifier(edge.src_name.to_owned()));
        let mut op = Operator::OutEdgeSeqScan {
            element_name: edge.name.to_string(),
            edge_label,
            src,
        };
        if !edge.predicates.is_empty() {
            op = Operator::PredicateFilter {
                source: Box::new(op),
                predicates: edge.predicates.clone(),
            };
        }
        if !edge.projections.is_empty() {
            op = Operator::Projection {
                source: Box::new(op),
                items: edge.projections.clone(),
            }
        }
        op
    }
}

#[cfg(test)]
mod test {
    use crate::parser::ast::Statement;
    use crate::parser::parser::Parser;

    use super::*;

    #[test]
    fn test_build_select() {
        let stmts = Parser::parse_sql(
            concat!("SELECT a.label, b.label, c.prop1 FROM (b) <- [e] - (a) <- [e2] - (c)",
            "WHERE a.label = 'person' AND e.label == 'knows' AND a.id > '1' AND b.prop2 < 4 AND c.label > 'dog'"),
        ).unwrap();
        let mut planner = Planner::new();
        match &stmts[0] {
            Statement::Select {
                items,
                graph_pattern,
                condition,
            } => {
                let op = planner.build_select_query(items, graph_pattern, condition);
                println!("{:?}", op);
            }
            _ => panic!("error"),
        }
    }
}
