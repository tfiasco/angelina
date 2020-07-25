use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::parser::ast::Expr::{BinaryOp, UnaryOp};
use crate::parser::ast::{EdgeExpr, Expr, GraphPath, GraphTriplet, Statement, VertexExpr};
use crate::parser::ast::{GraphPattern, Value};
use crate::parser::keyword::Keyword;
use crate::parser::operator::{BinaryOperator, UnaryOperator};
use crate::parser::parser::ParserError::TokenizerError;
use crate::parser::tokenizer::{Token, Tokenizer};

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }

    pub fn parse_sql(sql: &str) -> Result<Vec<Statement>, ParserError> {
        let mut tokenizer = Tokenizer::new(sql);
        match tokenizer.tokenize() {
            Ok(tokens) => {
                let ws_skipped: Vec<Token> = tokens
                    .into_iter()
                    .filter(|t| match t {
                        Token::Whitespace(ws) => false,
                        _ => true,
                    })
                    .collect();
                println!("{:?}", ws_skipped);
                let mut parser = Self::new(ws_skipped);
                let mut stmts = Vec::new();
                while parser.peek_token() != Token::EOF {
                    let stmt = parser.parse_statement()?;
                    stmts.push(stmt);
                }
                Ok(stmts)
            }
            Err(e) => Err(ParserError::TokenizerError(e.message)),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.consume_token() {
            Token::Keyword(keyword) => {
                match keyword {
                    Keyword::SHOW => {
                        return if self
                            .match_and_consume_keywords(&[Keyword::VERTEX, Keyword::LABEL])
                        {
                            Ok(Statement::ShowVertexLabels)
                        } else if self.match_and_consume_keywords(&[Keyword::EDGE, Keyword::LABEL])
                        {
                            Ok(Statement::ShowEdgeLabels)
                        } else if self
                            .match_and_consume_keywords(&[Keyword::PROPERTY, Keyword::KEY])
                        {
                            Ok(Statement::ShowPropertyKeys)
                        } else {
                            Err(self.parser_error("unknown query".to_string()))
                        };
                    }
                    Keyword::SELECT => self.parse_select(),
                    Keyword::INSERT => self.parse_insert(),
                    // Keyword::UPDATE => self.parse_update(),
                    // Keyword::DELETE => self.parse_delete(),
                    Keyword::CREATE => self.parse_create(),
                    Keyword::DROP => self.parse_drop(),
                    _ => Err(self.parser_error(format!("Unexpected keyword `{}`", keyword))),
                }
            }
            token => Err(self.expect("Keyword", token)),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, ParserError> {
        let exprs = self.parse_separated(&Token::Comma, |parser| parser.parse_expr())?;
        let from = if self.match_and_consume_token(&Token::Keyword(Keyword::FROM)) {
            self.parse_graph_pattern()?
        } else {
            GraphPattern { triplets: vec![] }
        };
        let condition = if self.match_and_consume_token(&Token::Keyword(Keyword::WHERE)) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Statement::Select {
            items: exprs,
            from,
            condition,
        })
    }

    fn parse_graph_pattern(&mut self) -> Result<GraphPattern, ParserError> {
        let mut triplets = vec![];
        let mut curr = Box::new(self.parse_vertex_expr()?);

        loop {
            match self.consume_token() {
                Token::Minus => {
                    let edge = Box::new(self.parse_edge_expr()?);
                    self.check_match_and_consume_token(&Token::RightArrow);
                    let dst = Box::new(self.parse_vertex_expr()?);
                    triplets.push(GraphTriplet {
                        src: Box::clone(&curr),
                        edge,
                        dst: Box::clone(&dst),
                    });
                    curr = Box::clone(&dst);
                }
                Token::LeftArrow => {
                    let edge = Box::new(self.parse_edge_expr()?);
                    self.check_match_and_consume_token(&Token::Minus);
                    let src = Box::new(self.parse_vertex_expr()?);
                    triplets.push(GraphTriplet {
                        src: Box::clone(&src),
                        edge,
                        dst: Box::clone(&curr),
                    });
                    curr = Box::clone(&src);
                }
                Token::Comma => {
                    curr = Box::new(self.parse_vertex_expr()?);
                }
                _ => {
                    self.prev_token();
                    break;
                }
            }
        }
        Ok(GraphPattern { triplets })
    }

    fn parse_vertex_expr(&mut self) -> Result<Expr, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;
        let vertex_expr = match self.consume_token() {
            Token::Identifier(s) => Expr::Identifier(s),
            token => {
                self.prev_token();
                return Err(self.expect("Identifier", token));
            }
        };
        self.check_match_and_consume_token(&Token::RightParen)?;
        Ok(vertex_expr)
    }

    fn parse_edge_expr(&mut self) -> Result<Expr, ParserError> {
        self.check_match_and_consume_token(&Token::LeftBracket)?;
        let edge_expr = match self.consume_token() {
            Token::Identifier(s) => Expr::Identifier(s),
            token => {
                self.prev_token();
                return Err(self.expect("Identifier", token));
            }
        };
        self.check_match_and_consume_token(&Token::RightBracket)?;
        Ok(edge_expr)
    }

    fn parse_insert(&mut self) -> Result<Statement, ParserError> {
        if self.match_and_consume_token(&Token::Keyword(Keyword::VERTEX)) {
            self.parse_insert_vertex()
        } else if self.match_and_consume_token(&Token::Keyword(Keyword::EDGE)) {
            self.parse_insert_edge()
        } else {
            Err(self.expect("Keyword VERTEX or EDGE", self.peek_token()))
        }
    }

    fn parse_insert_vertex(&mut self) -> Result<Statement, ParserError> {
        let label = self.parse_expr()?;

        self.check_match_and_consume_token(&Token::Keyword(Keyword::PROPERTIES))?;

        let properties = self.parse_properties()?;

        self.check_match_and_consume_token(&Token::Keyword(Keyword::VALUES))?;

        let vertex_id = self.parse_vertex_id()?;

        self.check_match_and_consume_token(&Token::Colon)?;

        let values = self.parse_values()?;

        Ok(Statement::InsertVertex {
            label,
            properties,
            vertex_id,
            values,
        })
    }

    fn parse_insert_edge(&mut self) -> Result<Statement, ParserError> {
        let label = self.parse_expr()?;

        self.check_match_and_consume_token(&Token::Keyword(Keyword::PROPERTIES))?;

        let properties = self.parse_properties()?;

        self.check_match_and_consume_token(&Token::Keyword(Keyword::VALUES))?;

        let src_dst_id = self.parse_edge_vertices_id()?;

        self.check_match_and_consume_token(&Token::Colon)?;

        let values = self.parse_values()?;

        Ok(Statement::InsertEdge {
            label,
            properties,
            src_vertex_id: src_dst_id.0,
            dst_vertex_id: src_dst_id.1,
            values,
        })
    }

    fn parse_properties(&mut self) -> Result<Vec<String>, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let properties =
            self.parse_separated(&Token::Comma, |parser| match parser.peek_token() {
                Token::Identifier(s) => {
                    parser.consume_token();
                    Ok(s)
                }
                _ => Err(parser.expect("Identifiers", parser.peek_token())),
            })?;

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok(properties)
    }

    fn parse_vertex_id(&mut self) -> Result<Expr, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let vertex_id = self.parse_expr()?;

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok(vertex_id)
    }

    fn parse_edge_vertices_id(&mut self) -> Result<(Expr, Expr), ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let src_id = self.parse_expr()?;

        self.check_match_and_consume_token(&Token::RightArrow)?;

        let dst_id = self.parse_expr()?;

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok((src_id, dst_id))
    }

    fn parse_values(&mut self) -> Result<Vec<Expr>, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let values = self.parse_separated(&Token::Comma, |parser| match parser.peek_token() {
            Token::String(s) => {
                parser.consume_token();
                Ok(Expr::Value(Value::String(s)))
            }
            _ => Err(parser.parser_error("not impl".to_string())),
        })?;

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok(values)
    }

    fn parse_create(&mut self) -> Result<Statement, ParserError> {
        if self.match_and_consume_keywords(&[Keyword::VERTEX, Keyword::LABEL]) {
            self.parse_create_vertex_label()
        } else if self.match_and_consume_keywords(&[Keyword::EDGE, Keyword::LABEL]) {
            self.parse_create_edge_label()
        } else if self.match_and_consume_keywords(&[Keyword::PROPERTY, Keyword::KEY]) {
            self.parse_create_property_key()
        } else {
            Err(self.parser_error(format!("Unexpected token `{}`", self.peek_token())))
        }
    }

    fn parse_drop(&mut self) -> Result<Statement, ParserError> {
        if self.match_and_consume_keywords(&[Keyword::VERTEX, Keyword::LABEL]) {
            self.parse_drop_vertex_label()
        } else if self.match_and_consume_keywords(&[Keyword::EDGE, Keyword::LABEL]) {
            self.parse_drop_edge_label()
        } else if self.match_and_consume_keywords(&[Keyword::PROPERTY, Keyword::KEY]) {
            self.parse_drop_property_key()
        } else {
            Err(self.parser_error(format!("Unexpected token `{}`", self.peek_token())))
        }
    }

    fn parse_create_vertex_label(&mut self) -> Result<Statement, ParserError> {
        let next_token = self.peek_token();
        match next_token {
            Token::Identifier(ident) => {
                self.consume_token();
                Ok(Statement::CreateVertexLabel { name: ident })
            }
            _ => Err(self.expect("Identifier", next_token)),
        }
    }

    fn parse_create_edge_label(&mut self) -> Result<Statement, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let exprs = self.parse_separated(&Token::Comma, |parser| match parser.peek_token() {
            Token::Identifier(s) => {
                parser.consume_token();
                Ok(s)
            }
            _ => Err(parser.expect("Identifiers", parser.peek_token())),
        })?;

        if exprs.len() != 2 {
            return Err(self.parser_error("unexpected length of create edge clause".to_owned()));
        }

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok(Statement::CreateEdgeLabel {
            name: exprs[0].to_string(),
            multiplicity: exprs[1].to_string(),
        })
    }

    fn parse_create_property_key(&mut self) -> Result<Statement, ParserError> {
        self.check_match_and_consume_token(&Token::LeftParen)?;

        let exprs = self.parse_separated(&Token::Comma, |parser| match parser.peek_token() {
            Token::Identifier(s) => {
                parser.consume_token();
                Ok(s)
            }
            _ => Err(parser.expect("Identifiers", parser.peek_token())),
        })?;

        if exprs.len() != 2 {
            return Err(self.parser_error("unexpected length of create edge clause".to_owned()));
        }

        self.check_match_and_consume_token(&Token::RightParen)?;

        Ok(Statement::CreatePropertyKey {
            name: exprs[0].to_string(),
            cardinality: exprs[1].to_string(),
        })
    }

    fn parse_drop_vertex_label(&mut self) -> Result<Statement, ParserError> {
        let next_token = self.peek_token();
        match next_token {
            Token::Identifier(ident) => {
                self.consume_token();
                Ok(Statement::DropVertexLabel { name: ident })
            }
            _ => Err(self.expect("Identifier", next_token)),
        }
    }

    fn parse_drop_edge_label(&mut self) -> Result<Statement, ParserError> {
        let next_token = self.peek_token();
        match next_token {
            Token::Identifier(ident) => {
                self.consume_token();
                Ok(Statement::DropEdgeLabel { name: ident })
            }
            _ => Err(self.expect("Identifier", next_token)),
        }
    }

    fn parse_drop_property_key(&mut self) -> Result<Statement, ParserError> {
        let next_token = self.peek_token();
        match next_token {
            Token::Identifier(ident) => {
                self.consume_token();
                Ok(Statement::DropPropertyKey { name: ident })
            }
            _ => Err(self.expect("Identifier", next_token)),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_expr_tdop(BindingPower::Lowest)
    }

    fn parse_literal_value(&mut self) -> Result<Expr, ParserError> {
        match self.consume_token() {
            Token::String(s) => Ok(Expr::Value(Value::String(s))),
            Token::Number(n) => Ok(Expr::Value(Value::Number(n))),
            Token::Keyword(Keyword::TRUE) => Ok(Expr::Value(Value::Boolean(true))),
            Token::Keyword(Keyword::FALSE) => Ok(Expr::Value(Value::Boolean(false))),
            Token::Keyword(Keyword::NULL) => Ok(Expr::Value(Value::Null)),
            token => {
                self.prev_token();
                Err(self.expect("literal value", token))
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<Expr, ParserError> {
        match self.consume_token() {
            Token::Identifier(s) => Ok(Expr::Identifier(s)),
            token => {
                self.prev_token();
                Err(self.expect("identifier", token))
            }
        }
    }

    fn parse_identifiers_or_function(&mut self) -> Result<Expr, ParserError> {
        match self.consume_token() {
            Token::Identifier(s) => {
                return match self.consume_token() {
                    // function call. func(a, b, c)
                    Token::LeftParen => {
                        let arguments =
                            self.parse_separated(&Token::Comma, |parser| parser.parse_expr())?;
                        self.check_match_and_consume_token(&Token::RightParen)?;
                        Ok(Expr::Function {
                            func_name: s,
                            arguments,
                        })
                    }
                    // a.b.c, a.b.*
                    Token::Dot => {
                        let mut ident_vec = vec![s.to_string()];
                        loop {
                            match self.consume_token() {
                                Token::Identifier(ss) => ident_vec.push(ss),
                                Token::Keyword(Keyword::LABEL) => {
                                    return Ok(Expr::LabelExpr(Box::new(Expr::Identifier(
                                        s.to_string(),
                                    ))));
                                }
                                Token::Keyword(Keyword::ID) => {
                                    return Ok(Expr::IdExpr(Box::new(Expr::Identifier(
                                        s.to_string(),
                                    ))));
                                }
                                Token::Star => return Ok(Expr::CompoundWildcard(ident_vec)),
                                token => {
                                    self.prev_token();
                                    return Err(self.expect("Identifier or *", token));
                                }
                            }
                            if !self.match_and_consume_token(&Token::Dot) {
                                break;
                            }
                        }
                        Ok(Expr::CompoundIdentifier(ident_vec))
                    }
                    // single identifier
                    _ => {
                        self.prev_token();
                        Ok(Expr::Identifier(s))
                    }
                };
            }
            token => {
                self.prev_token();
                Err(self.expect("Identifier", token))
            }
        }
    }

    fn parse_expr_tdop(&mut self, rbp: BindingPower) -> Result<Expr, ParserError> {
        let mut expr = self.parse_prefix()?;
        loop {
            let lbp = self.get_binding_power();
            if rbp >= lbp {
                break;
            }
            expr = self.parse_infix(expr, lbp)?;
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParserError> {
        if self.match_and_consume_token(&Token::Star) {
            return Ok(Expr::Wildcard);
        }

        if let Ok(expr) = self.parse_literal_value() {
            return Ok(expr);
        }

        if let Ok(expr) = self.parse_identifiers_or_function() {
            return Ok(expr);
        }

        if let Ok(expr) = self.parse_unary_op() {
            return Ok(expr);
        }

        if self.match_and_consume_token(&Token::LeftParen) {
            let nested_expr = Expr::Nested(Box::new(self.parse_expr()?));
            self.check_match_and_consume_token(&Token::RightParen)?;
            return Ok(nested_expr);
        }

        Err(self.parser_error("syntax error".to_string()))
    }

    fn parse_unary_op(&mut self) -> Result<Expr, ParserError> {
        if let Some(op) = UnaryOperator::from_token(&self.peek_token()) {
            self.consume_token();
            return Ok(UnaryOp {
                op,
                expr: Box::new(self.parse_expr_tdop(op.get_binding_power())?),
            });
        }

        Err(self.parser_error("Not a unary op".to_string()))
    }

    fn get_binding_power(&mut self) -> BindingPower {
        match BinaryOperator::from_token(&self.peek_token()) {
            Some(op) => op.get_binding_power(),
            _ => BindingPower::Lowest,
        }
    }

    fn parse_infix(&mut self, expr: Expr, lbp: BindingPower) -> Result<Expr, ParserError> {
        if let Some(op) = BinaryOperator::from_token(&self.peek_token()) {
            self.consume_token();
            return Ok(Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(self.parse_expr_tdop(lbp)?),
            });
        }
        Err(self.parser_error("not impl".to_string()))
    }

    fn parse_separated<T, F>(
        &mut self,
        separator: &Token,
        parse_func: F,
    ) -> Result<Vec<T>, ParserError>
    where
        F: Fn(&mut Parser) -> Result<T, ParserError>,
    {
        let mut values = vec![];
        loop {
            values.push(parse_func(self)?);
            if !self.match_and_consume_token(separator) {
                break;
            }
        }
        Ok(values)
    }

    fn match_and_consume_keywords(&mut self, keywords: &[Keyword]) -> bool {
        let tokens: Vec<Token> = keywords
            .to_owned()
            .into_iter()
            .map(|kw| Token::Keyword(kw))
            .collect();
        self.match_and_consume_tokens(&tokens)
    }

    fn match_and_consume_tokens(&mut self, tokens: &[Token]) -> bool {
        let n = tokens.len();
        let peek_tokens = self.peek_next_n_token(n);
        for i in 0..n {
            if peek_tokens[i] != tokens[i] {
                return false;
            }
        }
        self.consume_next_n_token(n);
        true
    }

    fn check_match_and_consume_token(&mut self, token: &Token) -> Result<(), ParserError> {
        if self.match_and_consume_token(token) {
            Ok(())
        } else {
            Err(self.expect(token, self.peek_token()))
        }
    }

    fn match_and_consume_token(&mut self, token: &Token) -> bool {
        if &self.peek_token() == token {
            self.consume_token();
            return true;
        }
        false
    }

    fn peek_next_n_token(&self, n: usize) -> Vec<Token> {
        self.tokens[self.index..self.index + n].to_vec()
    }

    fn peek_token(&self) -> Token {
        self.tokens[self.index].clone()
    }

    fn consume_next_n_token(&mut self, n: usize) -> Vec<Token> {
        let tokens = &self.tokens[self.index..self.index + n];
        self.index += n;
        tokens.to_vec()
    }

    fn consume_token(&mut self) -> Token {
        let token = &self.tokens[self.index];
        self.index += 1;
        token.clone()
    }

    fn prev_token(&mut self) -> Token {
        self.index -= 1;
        self.tokens[self.index].clone()
    }

    fn expect<T, U>(&self, expect: T, found: U) -> ParserError
    where
        T: Display,
        U: Display,
    {
        self.parser_error(format!("Expect `{}` but found `{}`", expect, found))
    }

    fn parser_error(&self, msg: String) -> ParserError {
        ParserError::ParserError(format!("{} at position {}", msg, self.index))
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum BindingPower {
    /// lowest binding power
    Lowest = 0,
    /// a AND b OR c ...
    AndOr = 20,
    /// a == b, a <= b ...
    Compare = 30,
    /// a + b, a - b ...
    PlusMinus = 40,
    /// a * b, a / b, a % b
    MultDiv = 50,
    /// NOT a
    Not = 60,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    TokenizerError(String),
    ParserError(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_schema_crud() {
        let stmts = Parser::parse_sql("CREATE VERTEX LABEL vertex_label").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("CREATE EDGE LABEL (edge_label, one2one)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("CREATE PROPERTY KEY (property_key, mono)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("DROP VERTEX LABEL vertex_label").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("DROP EDGE LABEL edge_label ").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("DROP PROPERTY KEY property_key ").unwrap();
        println!("{:?}", stmts);
    }

    #[test]
    fn test_simple_insert_vertex() {
        let stmts = Parser::parse_sql(
            "INSERT VERTEX 'vertex_label' PROPERTIES (prop1, prop2) VALUES ('vertex_id'):('value1', 'value2')",
        ).unwrap();
        println!("{:?}", stmts);
    }

    #[test]
    fn test_simple_insert_edge() {
        let stmts = Parser::parse_sql(
            "INSERT EDGE 'edge_label' PROPERTIES (prop1, prop2) VALUES ('vertex_id_1' -> 'vertex_id_2'):('value1', 'value2')"
        ).unwrap();
        println!("{:?}", stmts);
    }

    #[test]
    fn test_parse_expr() {
        let stmts = Parser::parse_sql("SELECT 1 + 2 + 3").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT -1 + 2 * 3 - 1").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT a > 3").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT NOT a.b AND (b OR c)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT 2 * (3 + 1)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT func(arg1, arg2, func2(a, b+1, True))").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT func(a.b.c, 2), a.b.*").unwrap();
        println!("{:?}", stmts);
    }

    #[test]
    fn test_parse_select() {
        let stmts = Parser::parse_sql("SELECT * FROM (a) - [e] -> (b)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT * FROM (b) <- [e] - (a)").unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql("SELECT * FROM (b) <- [e] - (a) - [e2] -> (c)").unwrap();
        println!("{:?}", stmts);
        let stmts =
            Parser::parse_sql("SELECT * FROM (b) <- [e] - (a) <- [e2] - (c), (b) - [e3] -> (c)")
                .unwrap();
        println!("{:?}", stmts);
        let stmts = Parser::parse_sql(
            "SELECT * FROM (b) <- [e] - (a) <- [e2] - (c) WHERE a.label = 'person' AND e.label == 'knows' AND a.id > '1' AND b.prop2 < 4 ",
        ).unwrap();
        println!("{:?}", stmts);
    }
}
