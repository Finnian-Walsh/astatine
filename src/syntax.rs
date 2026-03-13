use std::str::FromStr;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Modulo,

    Add,
    Subtract,

    ShiftL,
    ShiftR,
    BitAnd,
    Xor,
    BitOr,

    Equal,
    Unequal,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,

    And,
    Or,
}

#[derive(Clone, Debug, Error)]
#[error("Could not convert to binary operator")]
pub struct OperatorConversionError(String);

impl TryFrom<String> for BinaryOp {
    type Error = OperatorConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "*" => Self::Multiply,
            "/" => Self::Divide,
            "%" => Self::Modulo,

            "+" => Self::Add,
            "-" => Self::Subtract,

            "<<" => Self::ShiftL,
            ">>" => Self::ShiftR,
            "&" => Self::BitAnd,
            "^" => Self::Xor,
            "|" => Self::BitOr,

            "==" => Self::Equal,
            "!=" => Self::Unequal,
            "<" => Self::LessThan,
            ">" => Self::GreaterThan,
            "<=" => Self::LessOrEqual,
            ">=" => Self::GreaterOrEqual,

            "&&" => Self::And,
            "||" => Self::Or,

            _ => return Err(OperatorConversionError(value)),
        })
    }
}

impl TryFrom<&str> for BinaryOp {
    type Error = OperatorConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

#[allow(unused)]
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralKind {
    Char,
    Integer,
    Float,
    String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Const,
    Continue,
    Break,
    For,
    Func,
    Else,
    If,
    Let,
    Match,
    Return,
    Struct,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "continue" => Ok(Self::Continue),
            "break" => Ok(Self::Break),
            "for" => Ok(Self::For),
            "func" => Ok(Self::Func),
            "else" => Ok(Self::Else),
            "if" => Ok(Self::If),
            "let" => Ok(Self::Let),
            "match" => Ok(Self::Match),
            "return" => Ok(Self::Return),
            "struct" => Ok(Self::Struct),
            _ => Err(()),
        }
    }
}
