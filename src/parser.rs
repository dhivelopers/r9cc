use crate::errors::{CompileError, CompileErrorType, ParseError};
use crate::tokenizer::{Separator, TokenKind, Tokens};

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: TokenKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    fn new(kind: TokenKind, lhs: Option<Node>, rhs: Option<Node>) -> Self {
        Node {
            kind,
            lhs: lhs.map(Box::new),
            rhs: rhs.map(Box::new),
        }
    }

    pub fn expr(tokens: &mut Tokens) -> Result<Node, CompileError> {
        Self::equality(tokens)
    }

    fn equality(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::relational(tokens)?;

        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Eq => {
                    tokens.next();
                    node = Self::new(TokenKind::Eq, Some(node), Some(Self::relational(tokens)?));
                }
                TokenKind::NotEq => {
                    tokens.next();
                    node = Self::new(
                        TokenKind::NotEq,
                        Some(node),
                        Some(Self::relational(tokens)?),
                    );
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn relational(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::add(tokens)?;
        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Less => {
                    tokens.next();
                    node = Self::new(TokenKind::Less, Some(node), Some(Self::add(tokens)?));
                }
                TokenKind::LessEq => {
                    tokens.next();
                    node = Self::new(TokenKind::LessEq, Some(node), Some(Self::add(tokens)?));
                }
                TokenKind::Greater => {
                    tokens.next();
                    node = Self::new(TokenKind::Less, Some(Self::add(tokens)?), Some(node));
                }
                TokenKind::GreaterEq => {
                    tokens.next();
                    node = Self::new(TokenKind::LessEq, Some(Self::add(tokens)?), Some(node));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn add(tokens: &mut Tokens) -> Result<Node, CompileError> {
        // println!("{:#?}", tokens);
        let mut node = Self::mul(tokens)?;
        while let Some(token) = tokens.peek() {
            // println!("expr: {:#?}", token);
            match token.kind {
                TokenKind::Add => {
                    // println!("dbg! ok?");
                    tokens.next();
                    node = Self::new(TokenKind::Add, Some(node), Some(Self::mul(tokens)?));
                    // println!("{:#?}", node);
                }
                TokenKind::Sub => {
                    tokens.next();
                    node = Self::new(TokenKind::Sub, Some(node), Some(Self::mul(tokens)?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn mul(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::unary(tokens)?;
        while let Some(token) = tokens.peek() {
            // println!("mul: {:#?}", token);
            match token.kind {
                TokenKind::Mul => {
                    tokens.next();
                    node = Self::new(TokenKind::Mul, Some(node), Some(Self::unary(tokens)?));
                }
                TokenKind::Div => {
                    tokens.next();
                    node = Self::new(TokenKind::Div, Some(node), Some(Self::unary(tokens)?));
                }
                TokenKind::Number(_) => {
                    return Err(CompileError {
                        error_type: CompileErrorType::Parsing(ParseError::CannotParse),
                        pos: Some(token.span.clone()),
                    })
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn unary(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let result;
        // println!("{:#?}", result);
        if let Some(token) = tokens.peek() {
            // println!("unary: {:#?}", token);
            match token.kind {
                TokenKind::Add => {
                    // println!("+ {:?}", token.span);
                    tokens.next();
                    result = Self::unary(tokens);
                }
                TokenKind::Sub => {
                    tokens.next();
                    result = Ok(Self::new(
                        TokenKind::Sub,
                        Some(Self::new(TokenKind::Number(0), None, None)),
                        Some(Self::unary(tokens)?),
                    ));
                }
                _ => {
                    result = Self::primary(tokens);
                }
            }
        } else {
            return Err(CompileError {
                error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
                pos: None,
            });
        }
        result
    }

    fn primary(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let node;
        if let Some(token) = tokens.peek() {
            // println!("primary: {:#?}", token);
            let span = &token.span;
            // println!("{:?}", token);
            if token.kind == TokenKind::Sep(Separator::RoundBracketL) {
                tokens.next();
                node = Self::expr(tokens)?;
                if let Some(token) = tokens.peek() {
                    let span = &token.span;
                    if token.kind != TokenKind::Sep(Separator::RoundBracketR) {
                        return Err(CompileError {
                            error_type: CompileErrorType::Parsing(
                                ParseError::NotFoundRoundBracketR,
                            ),
                            pos: Some(span.clone()),
                        });
                    } else {
                        tokens.next();
                    }
                } else {
                    return Err(CompileError {
                        error_type: CompileErrorType::Parsing(ParseError::NotFoundRoundBracketR),
                        pos: None,
                    });
                }
            } else if let TokenKind::Number(num) = token.kind {
                tokens.next();
                node = Self::new(TokenKind::Number(num), None, None);
            } else {
                return Err(CompileError {
                    error_type: CompileErrorType::Parsing(ParseError::NotNumber),
                    pos: Some(span.clone()),
                });
            }
        } else {
            return Err(CompileError {
                error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
                pos: None,
            });
        }
        Ok(node)
    }
}
