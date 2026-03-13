use std::{iter::Peekable, slice::Iter};

use color_eyre::eyre::Result;
use thiserror::Error;

use crate::lexer::Token;
use crate::syntax::{BinaryOp, Bracket, Keyword, LiteralKind};

#[derive(Debug)]
pub enum Expression {
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
    },
    Declaration {
        name: String,
        value: Box<Expression>,
    },
    Identifier(String),
    Index {
        container: Box<Expression>,
        idx: Box<Expression>,
    },
    Literal {
        kind: LiteralKind,
        value: String,
    },
}

#[derive(Debug)]
pub struct Argument {
    value: String,
    tp: String,
}

#[derive(Debug)]
pub enum Node {
    Function {
        name: String,
        args: Vec<Argument>,
        statements: Vec<Expression>,
    },
    ConstDecl {
        name: String,
        value: Box<Expression>,
    },
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    nodes: Vec<Node>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Expected a token (context: {0})")]
    ExpectedToken(&'static str),

    #[error("Invalid expression (found `{0:?}`)")]
    InvalidExpression(Expression),

    // TODO: implement display for token, keyword
    #[error("Unexpected token: `{0:?}`; expected {1}")]
    UnexpectedToken(Token, &'static str),

    #[error("Unexpected keyword: `{0:?}`")]
    UnexpectedKeyword(Keyword),
}

trait OptionExt<T> {
    fn or_expected(self, context: &'static str) -> Result<T, ParseError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_expected(self, context: &'static str) -> Result<T, ParseError> {
        self.ok_or(ParseError::ExpectedToken(context))
    }
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
                return Err(ParseError::UnexpectedToken(
                    token.clone(),
                    "a keyword at the global scope",
                ));
            };

            let node = match keyword {
                Keyword::Const => self.parse_constant()?,
                Keyword::Func => self.parse_func()?,
                _ => return Err(ParseError::UnexpectedKeyword(keyword.clone())),
            };

            self.nodes.push(node);
        }

        Ok(self.nodes)
    }

    fn parse_declaration(&mut self) -> Result<Expression, ParseError> {
        self.tokens.next().or_expected("skipping declaration kw")?;

        let name_token = self
            .tokens
            .next()
            .or_expected("taking identifier for declaration")?;

        let Token::Identifier(name) = name_token else {
            return Err(ParseError::UnexpectedToken(
                name_token.clone(),
                "an identifier for the name of the declaration",
            ));
        };

        let assignment_tok = self.tokens.next().or_expected("assignment token needed")?;
        if !matches!(assignment_tok, Token::Equals) {
            return Err(ParseError::UnexpectedToken(
                assignment_tok.clone(),
                "an equals sign for assignment",
            ));
        }

        let expr = self
            .parse_expr(|token| {
                if matches!(token, Token::Semicolon) {
                    Some(token.clone())
                } else {
                    None
                }
            })?
            .0;

        Ok(Expression::Declaration {
            name: name.to_string(),
            value: Box::new(expr),
        })
    }

    fn parse_expr(
        &mut self,
        end_predicate: fn(&Token) -> Option<Token>,
    ) -> Result<(Expression, Token), ParseError> {
        // operand expected
        let mut expression = {
            let first_token = self
                .tokens
                .next()
                .or_expected("first token (for expression)")?;

            match first_token {
                Token::Identifier(ident) => Expression::Identifier(ident.to_string()),
                Token::Literal { kind, value } => Expression::Literal {
                    kind: kind.clone(),
                    value: value.to_string(),
                },
                // TODO: implement tuples
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        (*first_token).clone(),
                        "a literal or identifier for an expression",
                    ));
                }
            }
        };

        let mut next_token = self
            .tokens
            .next()
            .or_expected("next token expected in expression")?;

        let terminator = loop {
            if let Some(terminator) = end_predicate(next_token) {
                break terminator;
            }

            match next_token {
                Token::Bracket(bracket) => match bracket {
                    Bracket::LeftParen => {
                        let mut args = vec![];

                        loop {
                            let (expr, terminator) = self.parse_expr(|token| {
                                if matches!(
                                    token,
                                    Token::Comma | Token::Bracket(Bracket::RightParen)
                                ) {
                                    Some(token.clone())
                                } else {
                                    None
                                }
                            })?;

                            args.push(expr);

                            if terminator == Token::Bracket(Bracket::RightParen) {
                                break;
                            }
                        }

                        // the expression needs to be able to continue
                        expression = Expression::Call {
                            function: Box::new(expression),
                            args,
                        };
                    }
                    Bracket::LeftSquare => todo!("implement indexing"),
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            next_token.clone(),
                            "a left bracket (parenthesis or curly)",
                        ));
                    }
                },
                Token::BinaryOp(op) => {
                    let (rhs, terminator) = self.parse_expr(end_predicate)?;

                    return Ok((
                        Expression::BinaryOperation(
                            Box::new(expression),
                            op.clone(),
                            Box::new(rhs),
                        ),
                        terminator,
                    ));
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        next_token.clone(),
                        "a binary operator or a bracket for an expression operation",
                    ));
                }
            }

            next_token = self.tokens.next().or_expected("getting next token")?;
        };

        Ok((expression, terminator))
        // operator or termination expected
    }

    fn parse_arg(&mut self) -> Result<Argument, ParseError> {
        // TODO: implement function arguments
        todo!("implement function arguments")
    }

    fn parse_func(&mut self) -> Result<Node, ParseError> {
        self.tokens.next().or_expected("skipping function kw")?; // discard function marker
        let name_tok = self.tokens.next().or_expected("function name required")?;

        let Token::Identifier(name) = name_tok else {
            return Err(ParseError::UnexpectedToken(
                name_tok.clone(),
                "an identifier for a function name",
            ));
        };

        // TODO: implement generics (not for a long time though)
        let left_paren_tok = self
            .tokens
            .next()
            .or_expected("left paren token expected")?;
        if !matches!(left_paren_tok, Token::Bracket(Bracket::LeftParen)) {
            return Err(ParseError::UnexpectedToken(
                left_paren_tok.clone(),
                "a left parenthesis for a function",
            ));
        }

        let mut args = vec![];

        while !matches!(
            self.tokens
                .peek()
                .or_expected("token needed to compare to right paren")?,
            Token::Bracket(Bracket::RightParen)
        ) {
            args.push(self.parse_arg()?);
        }

        self.tokens.next().or_expected("skipping right paren")?;

        let mut statements = vec![];

        let left_curly_tok = self
            .tokens
            .next()
            .or_expected("left curly token expected")?;
        if !matches!(left_curly_tok, Token::Bracket(Bracket::LeftCurly)) {
            return Err(ParseError::UnexpectedToken(
                left_curly_tok.clone(),
                "a left curly bracket for a function",
            ));
        }

        while !matches!(
            self.tokens
                .peek()
                .or_expected("token needed to compare to right curly")?,
            Token::Bracket(Bracket::RightCurly)
        ) {
            statements.push(
                match self
                    .tokens
                    .peek()
                    .or_expected("token needed for statement")?
                {
                    Token::Keyword(keyword) => match keyword {
                        Keyword::Let => self.parse_declaration()?,
                        _ => todo!("implement support for other keywords in statements"),
                    },
                    _ => todo!("implement other statements"),
                },
            )
        }

        self.tokens
            .next()
            .or_expected("right curly at the end of a function")?;

        Ok(Node::Function {
            name: name.to_string(),
            args,
            statements,
        })
    }

    fn parse_constant(&mut self) -> Result<Node, ParseError> {
        let expression = self.parse_declaration()?;

        let Expression::Declaration { name, value } = expression else {
            return Err(ParseError::InvalidExpression(expression));
        };

        Ok(Node::ConstDecl { name, value })
    }
}
