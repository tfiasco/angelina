use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::execution::executor::ExecutionError;
use crate::parser::ast::{Expr, GraphPattern, Statement, Value};
use crate::parser::operator::{BinaryOperator, UnaryOperator};

#[derive(Debug)]
pub struct Scope {
    pub vertices: HashMap<String, VertexPattern>,
    pub edges: HashMap<String, EdgePattern>,
    pub conditions: Vec<Expr>,
    pub select_items: Vec<Expr>,
    pub paths: Vec<(String, String, String)>,
}

#[derive(Debug, Clone)]
pub enum Comparator<T> {
    Eq(T),
    // Gt(T),
    // Lt(T),
    Gte(T),
    Lte(T),
}

#[derive(Debug, Clone)]
pub struct VertexPattern {
    pub name: String,
    pub label: Option<String>,
    pub id: Vec<Comparator<Expr>>,
    pub predicates: Vec<Expr>,
    pub projections: Vec<Expr>,
}

impl VertexPattern {
    pub fn new(name: &str) -> Self {
        VertexPattern {
            name: name.to_owned(),
            label: None,
            id: vec![],
            predicates: vec![],
            projections: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgePattern {
    pub name: String,
    pub label: Option<String>,
    pub src_name: String,
    pub dst_name: String,
    pub predicates: Vec<Expr>,
    pub projections: Vec<Expr>,
    pub num: (u32, u32),
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            vertices: Default::default(),
            edges: Default::default(),
            conditions: vec![],
            select_items: vec![],
            paths: vec![],
        }
    }

    pub fn parse_select_query(
        &mut self,
        items: &Vec<Expr>,
        graph_pattern: &GraphPattern,
        condition: &Option<Expr>,
    ) {
        self.parse_graph_pattern(graph_pattern);
        match condition {
            Some(cond) => self.parse_condition(&cond),
            None => {}
        }
        self.parse_select_items(items);
    }

    fn parse_graph_pattern(&mut self, graph_pattern: &GraphPattern) {
        for triplet in &graph_pattern.triplets {
            let src_name = match triplet.src.as_ref() {
                Expr::Identifier(s) => {
                    self.vertices.insert(s.to_owned(), VertexPattern::new(&s));
                    s.to_owned()
                }
                _ => panic!("vertex should be identifier"),
            };
            let dst_name = match triplet.dst.as_ref() {
                Expr::Identifier(s) => {
                    self.vertices.insert(s.to_owned(), VertexPattern::new(&s));
                    s.to_owned()
                }
                _ => panic!("vertex should be identifier"),
            };
            match triplet.edge.as_ref() {
                Expr::Identifier(edge_name) => {
                    self.edges.insert(
                        edge_name.to_owned(),
                        EdgePattern {
                            name: edge_name.to_owned(),
                            label: None,
                            src_name: src_name.to_owned(),
                            dst_name: dst_name.to_owned(),
                            predicates: vec![],
                            projections: vec![],
                            num: (0, 0),
                        },
                    );
                    self.paths.push((
                        src_name.to_owned(),
                        edge_name.to_owned(),
                        dst_name.to_owned(),
                    ))
                }
                _ => panic!("edge should be identifier"),
            };
        }
    }

    fn parse_condition(&mut self, condition: &Expr) {
        match condition {
            Expr::Function { .. } => self.push_conditions_into_scope(condition),
            Expr::UnaryOp { .. } => self.push_conditions_into_scope(condition),
            Expr::BinaryOp { op, left, right } => match op {
                op if op == &BinaryOperator::And || op == &BinaryOperator::Or => {
                    self.parse_condition(left);
                    self.parse_condition(right);
                }
                _ => {
                    let mut expr_type = None;
                    let mut element_name = &"".to_string();
                    let mut value = "".to_string();
                    let mut comp = None;
                    match (left.as_ref(), right.as_ref()) {
                        (Expr::LabelExpr(name), Expr::Value(Value::String(v))) => {
                            element_name = name;
                            value = v.to_owned();
                            expr_type = Some("label")
                        }
                        (Expr::Value(Value::String(v)), Expr::LabelExpr(name)) => {
                            element_name = name;
                            value = v.to_owned();
                            expr_type = Some("label")
                        }
                        (Expr::IdExpr(name), Expr::Value(Value::String(v))) => {
                            element_name = name;
                            value = v.to_owned();
                            expr_type = Some("id")
                        }
                        (Expr::Value(Value::String(v)), Expr::IdExpr(name)) => {
                            element_name = name;
                            value = v.to_owned();
                            expr_type = Some("id")
                        }
                        _ => {}
                    }
                    let mut neq = vec![];
                    match op {
                        BinaryOperator::Eq => {
                            comp = Some(Comparator::Eq(Expr::Value(Value::String(value))));
                        }
                        BinaryOperator::Gt => {
                            comp = Some(Comparator::Gte(Expr::Value(Value::String(
                                value.to_owned(),
                            ))));
                            neq.push(Expr::Value(Value::String(value.to_owned())));
                        }
                        BinaryOperator::Lt => {
                            comp = Some(Comparator::Lte(Expr::Value(Value::String(
                                value.to_owned(),
                            ))));
                            neq.push(Expr::Value(Value::String(value.to_owned())));
                        }
                        BinaryOperator::Gte => {
                            comp = Some(Comparator::Gte(Expr::Value(Value::String(value))));
                        }
                        BinaryOperator::Lte => {
                            comp = Some(Comparator::Lte(Expr::Value(Value::String(value))));
                        }
                        _ => {}
                    }
                    match expr_type {
                        Some("label") => {
                            if let Some(Comparator::Eq(Expr::Value(Value::String(value)))) = comp {
                                if self.vertices.contains_key(element_name) {
                                    self.vertices.get_mut(element_name).unwrap().label = Some(value)
                                } else if self.edges.contains_key(element_name) {
                                    self.edges.get_mut(element_name).unwrap().label = Some(value)
                                } else {
                                    panic!("no such element")
                                }
                            } else {
                                self.push_conditions_into_scope(condition)
                            }
                        }
                        Some("id") => {
                            if self.vertices.contains_key(element_name) {
                                self.vertices
                                    .get_mut(element_name)
                                    .unwrap()
                                    .id
                                    .push(comp.unwrap());
                                if !neq.is_empty() {
                                    neq.insert(0, Expr::IdExpr(element_name.to_owned()));
                                    self.push_conditions_into_scope(&Expr::UnaryOp {
                                        op: UnaryOperator::Not,
                                        expr: Box::new(Expr::Function {
                                            func_name: "in".to_string(),
                                            arguments: neq,
                                        }),
                                    })
                                }
                            } else {
                                panic!("no such element")
                            }
                        }
                        _ => self.push_conditions_into_scope(condition),
                    }
                }
            },
            Expr::Nested(expr) => self.parse_condition(expr),
            _ => panic!("unknown where condition."),
        }
    }

    fn parse_select_items(&mut self, items: &Vec<Expr>) {
        for item in items {
            self.select_items.push(item.clone());
        }
    }

    /// determine a expr is a element predicate or not.
    /// definition of `element predicate` is
    ///   - a bool expression which contains one single element (one vertex or one edge) of graph
    ///   - for a pattern `(a) - [e] -> (b)`, these are element predicates
    ///     - a.prop = 1, b.label != 'person', func(e.prop2) > 3
    ///   - these are not element predicates  (expression contains more than one elements)
    ///     - a.prop1 = b.prop2, func(a.prop1, e.prop2) < 2
    fn push_conditions_into_scope(&mut self, condition: &Expr) {
        let mut elements_in_func = vec![];
        self.collect_elements_in_graph(condition, &mut elements_in_func);
        if elements_in_func.len() == 1 {
            let element_name = &elements_in_func[0];
            if self.vertices.contains_key(element_name) {
                self.vertices
                    .get_mut(element_name)
                    .unwrap()
                    .predicates
                    .push(condition.clone())
            } else if self.edges.contains_key(element_name) {
                self.edges
                    .get_mut(element_name)
                    .unwrap()
                    .predicates
                    .push(condition.clone())
            } else {
                panic!("no such element")
            }
        } else {
            // TODO if elements_in_func is empty ?
            self.conditions.push(condition.clone())
        }
    }

    fn collect_elements_in_graph<'a>(
        &self,
        expr: &Expr,
        elements: &'a mut Vec<String>,
    ) -> &'a mut Vec<String> {
        match expr {
            Expr::Identifier(_) => {}
            Expr::Value(_) => {}
            Expr::CompoundIdentifier(idents) => {
                if idents.len() != 2 {
                    panic!("unknown identifiers")
                }

                let element_name = &idents[0];
                if self.is_graph_element(element_name) {
                    elements.push(element_name.to_owned());
                } else {
                    panic!("no such element")
                }
            }
            Expr::Wildcard => {}
            Expr::CompoundWildcard(idents) => {
                if idents.len() != 1 {
                    panic!("unknown identifiers")
                }

                let element_name = &idents[0];
                if self.is_graph_element(element_name) {
                    elements.push(element_name.to_owned());
                } else {
                    panic!("no such element")
                }
            }
            Expr::Function {
                func_name,
                arguments,
            } => {
                for arg in arguments {
                    self.collect_elements_in_graph(arg, elements);
                }
            }
            Expr::UnaryOp { op, expr } => {
                self.collect_elements_in_graph(expr, elements);
            }
            Expr::BinaryOp { op, left, right } => {
                self.collect_elements_in_graph(left, elements);
                self.collect_elements_in_graph(right, elements);
            }
            Expr::Nested(expr) => {
                self.collect_elements_in_graph(expr, elements);
            }
            Expr::LabelExpr(element_name) => {
                if self.is_graph_element(element_name) {
                    elements.push(element_name.to_owned());
                } else {
                    panic!("no such element")
                }
            }
            Expr::IdExpr(element_name) => {
                if self.is_graph_element(element_name) {
                    elements.push(element_name.to_owned());
                } else {
                    panic!("no such element")
                }
            }
        };
        elements
    }

    fn is_graph_element(&self, element_name: &str) -> bool {
        self.vertices.contains_key(element_name) || self.edges.contains_key(element_name)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::parser::Parser;

    use super::*;

    #[test]
    fn test_build_select() {
        let stmts = Parser::parse_sql(
            concat!("SELECT a.label, b.label, c.prop1 FROM (b) <- [e] - (a) <- [e2] - (c)",
            "WHERE a.label = 'person' AND e.label == 'knows' AND a.id > '1' AND b.prop2 < 4 AND c.label > 'dog'"),
        ).unwrap();
        let mut scope = Scope::new();
        match &stmts[0] {
            Statement::Select {
                items,
                graph_pattern,
                condition,
            } => {
                scope.parse_select_query(items, graph_pattern, condition);
                println!("{:?}", scope);
            }
            _ => panic!("error"),
        }
    }
}
