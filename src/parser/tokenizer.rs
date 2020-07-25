use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;
use std::string::ToString;

use crate::parser::keyword::Keyword;
use crate::parser::parser::ParserError::TokenizerError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    /// End of File
    EOF,
    /// keyword
    Keyword(Keyword),
    /// identifier
    Identifier(String),
    /// number literal
    Number(String),
    /// 'string', "string"
    String(String),
    /// space, tab, enter ...
    Whitespace(Whitespace),
    /// =
    Eq,
    /// ==
    DoubleEq,
    /// !=
    Neq,
    /// <
    Lt,
    /// >
    Gt,
    /// <=
    Lte,
    /// >=
    Gte,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// ,
    Comma,
    /// .
    Dot,
    /// :
    Colon,
    /// ;
    SemiColon,
    /// !
    Question,
    /// \
    Backslash,
    /// _
    UnderScore,
    /// &
    Ampersand,
    /// |
    Bar,
    /// ^
    Caret,
    /// $
    Dollar,
    /// #
    Sharp,
    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// <-
    LeftArrow,
    /// ->
    RightArrow,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::EOF => write!(f, "{}", "EOF"),
            Token::Keyword(keyword) => write!(f, "{}", keyword.to_string()),
            Token::Identifier(ident) => write!(f, "{}", ident),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "{}", s),
            Token::Whitespace(ws) => write!(f, "{}", ws.to_string()),
            Token::Eq => write!(f, "{}", "="),
            Token::DoubleEq => write!(f, "{}", "=="),
            Token::Neq => write!(f, "{}", "!="),
            Token::Lt => write!(f, "{}", "<"),
            Token::Gt => write!(f, "{}", ">"),
            Token::Lte => write!(f, "{}", "<="),
            Token::Gte => write!(f, "{}", ">="),
            Token::Plus => write!(f, "{}", "+"),
            Token::Minus => write!(f, "{}", "-"),
            Token::Star => write!(f, "{}", "*"),
            Token::Slash => write!(f, "{}", "/"),
            Token::Percent => write!(f, "{}", "%"),
            Token::Comma => write!(f, "{}", ","),
            Token::Dot => write!(f, "{}", "."),
            Token::Colon => write!(f, "{}", ":"),
            Token::SemiColon => write!(f, "{}", ";"),
            Token::Question => write!(f, "{}", "?"),
            Token::Backslash => write!(f, "{}", "\\"),
            Token::UnderScore => write!(f, "{}", "_"),
            Token::Ampersand => write!(f, "{}", "&"),
            Token::Bar => write!(f, "{}", "|"),
            Token::Caret => write!(f, "{}", "^"),
            Token::Dollar => write!(f, "{}", "$"),
            Token::Sharp => write!(f, "{}", "#"),
            Token::LeftParen => write!(f, "{}", "("),
            Token::RightParen => write!(f, "{}", ")"),
            Token::LeftBracket => write!(f, "{}", "["),
            Token::RightBracket => write!(f, "{}", "]"),
            Token::LeftBrace => write!(f, "{}", "{"),
            Token::RightBrace => write!(f, "{}", "}"),
            Token::LeftArrow => write!(f, "{}", "<-"),
            Token::RightArrow => write!(f, "{}", "->"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Whitespace {
    Space,
    Tab,
    Newline,
}

impl Display for Whitespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Whitespace::Space => write!(f, " "),
            Whitespace::Tab => write!(f, "\t"),
            Whitespace::Newline => write!(f, "\n"),
        }
    }
}

pub struct Tokenizer {
    pub query: String,
    pub line: u64,
    pub col: u64,
}

impl Tokenizer {
    pub fn new(query: &str) -> Tokenizer {
        Tokenizer {
            query: query.to_string(),
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizeError> {
        let mut chars = self.query.chars().peekable();

        let mut tokens = vec![];

        while let Some(token) = self.next_token(&mut chars)? {
            match &token {
                Token::Whitespace(Whitespace::Newline) => {
                    self.line += 1;
                    self.col = 1;
                }
                Token::Whitespace(Whitespace::Tab) => self.col += 4,
                Token::Keyword(s) => self.col += s.to_string().len() as u64,
                Token::Identifier(s) => self.col += s.len() as u64,
                Token::Number(s) => self.col += s.len() as u64,
                Token::String(s) => self.col += s.len() as u64 + 2,
                Token::DoubleEq | Token::Lte | Token::Gte | Token::Neq => self.col += 2,
                _ => self.col += 1,
            }
            tokens.push(token);
        }

        tokens.push(Token::EOF);

        Ok(tokens)
    }

    fn error<T>(&self, message: &str) -> Result<T, TokenizeError> {
        Err(TokenizeError {
            message: message.to_string(),
            line: self.line,
            col: self.col,
        })
    }

    fn next_token(&self, chars: &mut Peekable<Chars>) -> Result<Option<Token>, TokenizeError> {
        match chars.peek() {
            Some(&c) => match c {
                ' ' => Self::consume_token(chars, Token::Whitespace(Whitespace::Space)),
                '\t' => Self::consume_token(chars, Token::Whitespace(Whitespace::Tab)),
                '\n' => Self::consume_token(chars, Token::Whitespace(Whitespace::Newline)),
                '\r' => Self::consume_token_and_match_peek(
                    chars,
                    Token::Whitespace(Whitespace::Newline),
                    vec![('\n', Token::Whitespace(Whitespace::Newline))],
                ),
                '\'' => self.tokenize_quoted_string(chars, '\''),
                '"' => self.tokenize_quoted_string(chars, '"'),
                '(' => Self::consume_token(chars, Token::LeftParen),
                ')' => Self::consume_token(chars, Token::RightParen),
                '[' => Self::consume_token(chars, Token::LeftBracket),
                ']' => Self::consume_token(chars, Token::RightBracket),
                '{' => Self::consume_token(chars, Token::LeftBrace),
                '}' => Self::consume_token(chars, Token::RightBrace),
                '=' => Self::consume_token_and_match_peek(
                    chars,
                    Token::Eq,
                    vec![('=', Token::DoubleEq)],
                ),
                '!' => Self::consume_token_and_match_peek(
                    chars,
                    Token::Question,
                    vec![('=', Token::Neq)],
                ),
                '<' => Self::consume_token_and_match_peek(
                    chars,
                    Token::Lt,
                    vec![('=', Token::Lte), ('-', Token::LeftArrow)],
                ),
                '>' => {
                    Self::consume_token_and_match_peek(chars, Token::Gt, vec![('=', Token::Gte)])
                }
                '+' => Self::consume_token(chars, Token::Plus),
                '-' => Self::consume_token_and_match_peek(
                    chars,
                    Token::Minus,
                    vec![('>', Token::RightArrow)],
                ),
                '*' => Self::consume_token(chars, Token::Star),
                '/' => Self::consume_token(chars, Token::Slash),
                '%' => Self::consume_token(chars, Token::Percent),
                ',' => Self::consume_token(chars, Token::Comma),
                '.' => Self::consume_token(chars, Token::Dot),
                ':' => Self::consume_token(chars, Token::Colon),
                ';' => Self::consume_token(chars, Token::SemiColon),
                '\\' => Self::consume_token(chars, Token::Backslash),
                '_' => Self::consume_token(chars, Token::UnderScore),
                '&' => Self::consume_token(chars, Token::Ampersand),
                '|' => Self::consume_token(chars, Token::Bar),
                '^' => Self::consume_token(chars, Token::Caret),
                '$' => Self::consume_token(chars, Token::Dollar),
                '#' => Self::consume_token(chars, Token::Sharp),
                c if Self::is_identifier_start(c) => self.tokenize_identifier_or_keyword(chars),
                '0'..='9' => {
                    let s = Self::consume_while(chars, |x| match x {
                        '0'..='9' | '.' => true,
                        _ => false,
                    });
                    Ok(Some(Token::Number(s)))
                }
                _ => self.error("unexpected token!"),
            },
            None => Ok(None),
        }
    }

    fn is_identifier_start(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
    }

    fn is_identifier_char(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_'
    }

    fn tokenize_identifier_or_keyword(
        &self,
        chars: &mut Peekable<Chars>,
    ) -> Result<Option<Token>, TokenizeError> {
        let s = Self::consume_while(chars, |x| Self::is_identifier_char(x));

        match Keyword::from_str(&s.to_uppercase()) {
            Ok(kw) => Ok(Some(Token::Keyword(kw))),
            Err(e) => Ok(Some(Token::Identifier(s))),
        }
    }

    fn tokenize_quoted_string(
        &self,
        chars: &mut Peekable<Chars>,
        quote_char: char,
    ) -> Result<Option<Token>, TokenizeError> {
        chars.next();
        let mut s = String::new();
        while let Some(&c) = chars.peek() {
            chars.next();
            if c == '\\' && chars.peek().unwrap_or(&' ') == &quote_char {
                chars.next();
                s.push(quote_char);
            } else if c != quote_char {
                s.push(c);
            } else {
                return Ok(Some(Token::String(s)));
            }
        }
        self.error("EOF when matching string literal")
    }

    fn consume_token(
        chars: &mut Peekable<Chars>,
        token: Token,
    ) -> Result<Option<Token>, TokenizeError> {
        chars.next();
        Ok(Some(token))
    }

    fn consume_token_and_match_peek(
        chars: &mut Peekable<Chars>,
        token: Token,
        match_peek: Vec<(char, Token)>,
    ) -> Result<Option<Token>, TokenizeError> {
        chars.next();
        match chars.peek() {
            Some(&c) => {
                for (peek, peek_token) in match_peek {
                    if c == peek {
                        chars.next();
                        return Ok(Some(peek_token));
                    }
                }
                Ok(Some(token))
            }
            None => Ok(Some(token)),
        }
    }

    fn consume_while(chars: &mut Peekable<Chars>, predicate: impl Fn(char) -> bool) -> String {
        let mut s = String::new();
        while let Some(&c) = chars.peek() {
            if predicate(c) {
                chars.next();
                s.push(c);
            } else {
                break;
            }
        }
        s
    }
}

pub struct TokenizeError {
    pub message: String,
    pub line: u64,
    pub col: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_select() {
        let sql = "SELECT * FROM label1 Where a = 123 and b != '456'";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::Star,
                Token::Whitespace(Whitespace::Space),
                Token::Keyword(Keyword::FROM),
                Token::Whitespace(Whitespace::Space),
                Token::Identifier("label1".to_owned()),
                Token::Whitespace(Whitespace::Space),
                Token::Keyword(Keyword::WHERE),
                Token::Whitespace(Whitespace::Space),
                Token::Identifier("a".to_owned()),
                Token::Whitespace(Whitespace::Space),
                Token::Eq,
                Token::Whitespace(Whitespace::Space),
                Token::Number("123".to_owned()),
                Token::Whitespace(Whitespace::Space),
                Token::Keyword(Keyword::AND),
                Token::Whitespace(Whitespace::Space),
                Token::Identifier("b".to_owned()),
                Token::Whitespace(Whitespace::Space),
                Token::Neq,
                Token::Whitespace(Whitespace::Space),
                Token::String("456".to_owned()),
                Token::EOF,
            ],
            tokens
        );
    }

    #[test]
    fn test_function() {
        let sql = "SELECT func(prop1) FROM label1";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::Identifier("func".to_owned()),
                Token::LeftParen,
                Token::Identifier("prop1".to_owned()),
                Token::RightParen,
                Token::Whitespace(Whitespace::Space),
                Token::Keyword(Keyword::FROM),
                Token::Whitespace(Whitespace::Space),
                Token::Identifier("label1".to_owned()),
                Token::EOF,
            ],
            tokens
        );
    }

    #[test]
    fn test_string() {
        let sql = "SELECT 'test_string'";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::String("test_string".to_owned()),
                Token::EOF,
            ],
            tokens
        );
    }

    #[test]
    fn test_string2() {
        let sql = "SELECT \"test_string\"";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::String("test_string".to_owned()),
                Token::EOF,
            ],
            tokens
        );
    }

    #[test]
    fn test_string3() {
        let sql = "SELECT 'test\\'_\\'string'";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::String("test'_'string".to_owned()),
                Token::EOF,
            ],
            tokens
        );

        let sql = "SELECT \"test\\\"_\\\"string\"";
        let mut tokenizer = Tokenizer::new(&sql);
        let tokens = tokenizer.tokenize().unwrap_or_default();

        assert_eq!(
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Whitespace(Whitespace::Space),
                Token::String("test\"_\"string".to_owned()),
                Token::EOF,
            ],
            tokens
        );
    }
}
