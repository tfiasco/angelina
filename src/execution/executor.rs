use std::ops::Deref;
use std::rc::Rc;

use crate::datamodel::base::{Cardinality, EdgeMultiplicity};
use crate::datamodel::vertex::Vertex;
use crate::execution::memory::ExecutionMemory;
use crate::execution::operator::Operator;
use crate::execution::output::QueryOutput;
use crate::execution::planner::Planner;
use crate::execution::scope::{Comparator, Scope, VertexPattern};
use crate::handlers::edge_handler::EdgeHandler;
use crate::handlers::schema_handler::SchemaHandler;
use crate::handlers::sled_engine::SledEngine;
use crate::handlers::vertex_handler::VertexHandler;
use crate::parser::ast::{Expr, GraphPattern, Statement, Value};

pub struct QueryExecutor {
    schema_handler: SchemaHandler,
    vertex_handler: VertexHandler,
    edge_handler: EdgeHandler,
}

impl QueryExecutor {
    pub fn new(engine: Rc<Box<SledEngine>>) -> Self {
        QueryExecutor {
            schema_handler: SchemaHandler {
                engine: engine.clone(),
            },
            vertex_handler: VertexHandler {
                engine: engine.clone(),
            },
            edge_handler: EdgeHandler {
                engine: engine.clone(),
            },
        }
    }

    pub fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<QueryOutput, ExecutionError> {
        match statement {
            Statement::CreateVertexLabel { name } => {
                let id = self.schema_handler.create_vertex_label(&name);
                let output = QueryOutput {
                    columns: vec!["id".to_owned(), "name".to_owned(), "status".to_owned()],
                    items: Box::new(
                        vec![vec![id.to_string(), name.to_owned(), "CREATED".to_string()]]
                            .into_iter(),
                    ),
                };
                Ok(output)
            }
            Statement::CreateEdgeLabel { name, multiplicity } => {
                let id = self
                    .schema_handler
                    .create_edge_label(&name, EdgeMultiplicity::from(multiplicity.as_str()));
                let output = QueryOutput {
                    columns: vec!["id".to_owned(), "name".to_owned(), "status".to_owned()],
                    items: Box::new(
                        vec![vec![id.to_string(), name.to_owned(), "CREATED".to_string()]]
                            .into_iter(),
                    ),
                };
                Ok(output)
            }
            Statement::CreatePropertyKey { name, cardinality } => {
                let id = self
                    .schema_handler
                    .create_property_key(&name, Cardinality::from(cardinality.as_str()));
                let output = QueryOutput {
                    columns: vec!["id".to_owned(), "name".to_owned(), "status".to_owned()],
                    items: Box::new(
                        vec![vec![id.to_string(), name.to_owned(), "CREATED".to_string()]]
                            .into_iter(),
                    ),
                };
                Ok(output)
            }
            Statement::ShowVertexLabels => Ok(QueryOutput {
                columns: vec!["id".to_owned(), "name".to_owned()],
                items: Box::new(
                    self.schema_handler
                        .get_vertex_labels()
                        .into_iter()
                        .map(|label| vec![label.id.to_string(), label.name]),
                ),
            }),
            Statement::ShowEdgeLabels => {
                Ok(QueryOutput {
                    columns: vec![
                        "id".to_owned(),
                        "name".to_owned(),
                        "multiplicity".to_owned(),
                    ],
                    items: Box::new(self.schema_handler.get_edge_labels().into_iter().map(
                        |label| {
                            vec![
                                label.id.to_string(),
                                label.name,
                                label.multiplicity.to_string(),
                            ]
                        },
                    )),
                })
            }
            Statement::ShowPropertyKeys => {
                Ok(QueryOutput {
                    columns: vec!["id".to_owned(), "name".to_owned(), "cardinality".to_owned()],
                    items: Box::new(self.schema_handler.get_property_keys().into_iter().map(
                        |label| {
                            vec![
                                label.id.to_string(),
                                label.name,
                                label.cardinality.to_string(),
                            ]
                        },
                    )),
                })
            }
            Statement::InsertVertex {
                label,
                properties,
                vertex_id,
                values,
            } => self.execute_insert_vertex(label, properties, vertex_id, values),
            Statement::Select {
                items,
                graph_pattern,
                condition,
            } => self.execute_select(items, graph_pattern, condition),
            _ => return Err(self.execute_error("not impl".to_string())),
        }
    }

    fn execute_select(
        &mut self,
        items: &Vec<Expr>,
        graph_pattern: &GraphPattern,
        condition: &Option<Expr>,
    ) -> Result<QueryOutput, ExecutionError> {
        let mut planner = Planner::new();
        let op = planner.build_select_query(items, graph_pattern, condition);
        // self.execute_operator(&op, &mut ExecutionMemory::new())?;
        Err(self.execute_error("not impl".to_string()))
    }

    fn execute_operator(
        &mut self,
        operator: &Operator,
        memory: &mut ExecutionMemory,
    ) -> Result<Box<dyn Iterator<Item = Vec<String>>>, ExecutionError> {
        match operator {
            _ => panic!("todo"),
        }
        Err(self.execute_error("not impl".to_string()))
    }

    fn execute_insert_vertex(
        &self,
        label: &Expr,
        properties: &Vec<String>,
        vertex_id: &Expr,
        values: &Vec<Expr>,
    ) -> Result<QueryOutput, ExecutionError> {
        let label_name = self.parse_label_name(label)?;
        let vid = self.parse_vertex_id(vertex_id)?;
        let props = properties
            .into_iter()
            .map(|name| self.schema_handler.get_property_key_by_name(name).unwrap());
        let values = values
            .into_iter()
            .map(|expr| self.execute_expr(expr).unwrap());
        return match self.schema_handler.get_vertex_label_by_name(&label_name) {
            Some(label) => {
                let mut vertex = self.vertex_handler.create_vertex(&vid, label.id);
                props.zip(values).for_each(|(prop, value)| {
                    self.vertex_handler.add_property(&mut vertex, &prop, &value)
                });
                Ok(QueryOutput {
                    columns: vec!["CREATED".to_string()],
                    items: Box::new(vec![vec!["1".to_string()]].into_iter()),
                })
            }
            None => Err(self.execute_error(format!("No Vertex Label named {}", label_name))),
        };
    }

    fn execute_expr(&self, expr: &Expr) -> Result<String, ExecutionError> {
        match expr {
            Expr::Value(Value::String(s)) => Ok(s.to_string()),
            _ => Err(self.execute_error("not impl".to_string())),
        }
    }

    fn parse_label_name(&self, label: &Expr) -> Result<String, ExecutionError> {
        match label {
            Expr::Identifier(s) => Ok(s.to_string()),
            _ => Err(self.execute_error("not impl".to_string())),
        }
    }

    fn parse_vertex_id(&self, vertex_id: &Expr) -> Result<String, ExecutionError> {
        match vertex_id {
            Expr::Value(Value::String(s)) => Ok(s.to_string()),
            _ => Err(self.execute_error("not impl. only string support".to_string())),
        }
    }

    fn execute_error(&self, msg: String) -> ExecutionError {
        ExecutionError { msg }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    msg: String,
}

#[cfg(test)]
mod test {
    use crate::parser::parser::Parser;

    use super::*;

    fn print_output(output: QueryOutput) {
        println!("--------------");
        println!("{:?}", output.columns);
        output.items.for_each(|x| println!("{:?}", x));
        println!("--------------");
    }

    #[test]
    fn test_create_schema() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));
        let mut qe = QueryExecutor::new(engine.clone());

        let stmt = &Parser::parse_sql("CREATE VERTEX LABEL vertex_label").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("CREATE EDGE LABEL (edge_label, one2one)").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("CREATE PROPERTY KEY (property_key, single)").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("SHOW VERTEX LABEL").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("SHOW EDGE LABEL").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("SHOW PROPERTY KEY").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);
    }

    #[test]
    fn test_insert_vertex() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));
        let mut qe = QueryExecutor::new(engine.clone());

        let stmt = &Parser::parse_sql("CREATE VERTEX LABEL vertex_label").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql("CREATE PROPERTY KEY (prop1, single)").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);
        let stmt = &Parser::parse_sql("CREATE PROPERTY KEY (prop2, single)").unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);

        let stmt = &Parser::parse_sql(
            "INSERT VERTEX vertex_label PROPERTIES (prop1, prop2) VALUES ('id1'):('v1', 'v2') ",
        )
        .unwrap()[0];
        let output = qe.execute_statement(stmt).unwrap();
        print_output(output);
    }
}
