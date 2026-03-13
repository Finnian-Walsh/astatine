use std::{iter::Peekable, slice::Iter};

use color_eyre::eyre::Result;
use thiserror::Error;

use crate::lexer::Token;
use crate::syntax::{BinaryOp, Keyword, LiteralKind};

pub struct Declaration {
    name: String,
    value: Expression,
}

pub enum Expression {
    Call { args: Vec<Expression> },
    Literal { value: String, kind: LiteralKind },
    Symbol(String),
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
}

pub enum Node {
    Function(Vec<Node>),
    Declaration(Declaration),
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    nodes: Vec<Node>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    // TODO: implement display for token, keyword
    #[error("Unexpected token: `{0:?}`")]
    UnexpectedToken(Token),

    #[error("Unexpected keyword: `{0:?}`")]
    UnexpectedKeyword(Keyword),
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
            nodes: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Node>, ParseError> {
        while let Some(&token) = self.tokens.peek() {
            let Token::Keyword(keyword) = token else {
                return Err(ParseError::UnexpectedToken(token.clone()));
            };

            match keyword {
                Keyword::Const => self.parse_constant()?,
                Keyword::Func => self.parse_func()?,
                _ => return Err(ParseError::UnexpectedKeyword(keyword.clone())),
            };
        }

        Ok(self.nodes)
    }

    fn parse_expr(end_predicate: fn(Token) -> bool) -> Result<Expression, ParseError> {
        todo!()
    }

    // TODO: implement 
    fn parse_func(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_constant(&mut self) -> Result<(), ParseError> {
        todo!()
    }

}
