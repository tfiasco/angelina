use crate::parser::keyword::Keyword;
use crate::parser::parser::BindingPower;
use crate::parser::tokenizer::Token;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
}

impl UnaryOperator {
    pub fn from_token(token: &Token) -> Option<UnaryOperator> {
        match token {
            Token::Plus => Some(UnaryOperator::Plus),
            Token::Minus => Some(UnaryOperator::Minus),
            Token::Keyword(Keyword::NOT) => Some(UnaryOperator::Not),
            _ => None,
        }
    }

    pub fn get_binding_power(&self) -> BindingPower {
        match self {
            Self::Plus | Self::Minus => BindingPower::PlusMinus,
            UnaryOperator::Not => BindingPower::Not,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    NotEq,
    And,
    Or,
    Like,
}

impl BinaryOperator {
    pub fn from_token(token: &Token) -> Option<BinaryOperator> {
        match token {
            Token::Plus => Some(BinaryOperator::Plus),
            Token::Minus => Some(BinaryOperator::Minus),
            Token::Star => Some(BinaryOperator::Multiply),
            Token::Slash => Some(BinaryOperator::Divide),
            Token::Percent => Some(BinaryOperator::Modulus),
            Token::Gt => Some(BinaryOperator::Gt),
            Token::Lt => Some(BinaryOperator::Lt),
            Token::Gte => Some(BinaryOperator::Gte),
            Token::Lte => Some(BinaryOperator::Lte),
            Token::Eq | Token::DoubleEq => Some(BinaryOperator::Eq),
            Token::Neq => Some(BinaryOperator::NotEq),
            Token::Keyword(Keyword::AND) => Some(BinaryOperator::And),
            Token::Keyword(Keyword::OR) => Some(BinaryOperator::Or),
            _ => None,
        }
    }

    pub fn get_binding_power(&self) -> BindingPower {
        match self {
            Self::Plus | Self::Minus => BindingPower::PlusMinus,
            Self::Multiply | Self::Divide | Self::Modulus => BindingPower::MultDiv,
            Self::Gt | Self::Lt | Self::Gte | Self::Lte | Self::Eq | Self::NotEq => {
                BindingPower::Compare
            }
            Self::And | Self::Or => BindingPower::AndOr,
            _ => BindingPower::Lowest,
        }
    }
}
