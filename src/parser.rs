use crate::errors::{CompileError, CompileErrorType, ParseError};
use crate::tokenizer::{Separator, TokenKind, Tokens};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeKind {
    Var(usize), // variable (converted ident) // usize is offset
    Number(i64),
    Add,
    Sub,
    Mul,
    Div,
    Eq,     // '=='
    NotEq,  // '!='
    Less,   // '<'
    LessEq, // '<='
    Assign, // '='
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    fn new(kind: NodeKind, lhs: Option<Node>, rhs: Option<Node>) -> Self {
        Node {
            kind,
            lhs: lhs.map(Box::new),
            rhs: rhs.map(Box::new),
        }
    }

    fn offset(s: &str) -> usize {
        let offset = ((s.chars().next().unwrap() as usize) - ('a' as usize) + 1) * 8;
        offset
    }

    pub fn program(tokens: &mut Tokens) -> Result<Vec<Node>, CompileError> {
        let mut code = vec![];
        while tokens.peek().is_some() {
            code.push(Self::stmt(tokens)?);
        }
        Ok(code)
    }

    fn stmt(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let node = Self::expr(tokens)?;
        if let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Sep(Separator::SemiColon) => {
                    tokens.next();
                    Ok(node)
                }
                _ => Err(CompileError {
                    error_type: CompileErrorType::Parsing(ParseError::NeedSemiColon),
                    pos: Some(token.span.clone()),
                }),
            }
        } else {
            Err(CompileError {
                error_type: CompileErrorType::Parsing(ParseError::NeedSemiColon),
                pos: None,
            })
        }
    }

    fn expr(tokens: &mut Tokens) -> Result<Node, CompileError> {
        Self::assign(tokens)
    }

    fn assign(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::equality(tokens)?;
        if let Some(token) = tokens.peek() {
            if token.kind == TokenKind::Assign {
                tokens.next();
                node = Self::new(NodeKind::Assign, Some(node), Some(Self::assign(tokens)?));
            }
        }
        Ok(node)
    }

    fn equality(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::relational(tokens)?;

        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Eq => {
                    tokens.next();
                    node = Self::new(NodeKind::Eq, Some(node), Some(Self::relational(tokens)?));
                }
                TokenKind::NotEq => {
                    tokens.next();
                    node = Self::new(NodeKind::NotEq, Some(node), Some(Self::relational(tokens)?));
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
                    node = Self::new(NodeKind::Less, Some(node), Some(Self::add(tokens)?));
                }
                TokenKind::LessEq => {
                    tokens.next();
                    node = Self::new(NodeKind::LessEq, Some(node), Some(Self::add(tokens)?));
                }
                TokenKind::Greater => {
                    tokens.next();
                    node = Self::new(NodeKind::Less, Some(Self::add(tokens)?), Some(node));
                }
                TokenKind::GreaterEq => {
                    tokens.next();
                    node = Self::new(NodeKind::LessEq, Some(Self::add(tokens)?), Some(node));
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
                    node = Self::new(NodeKind::Add, Some(node), Some(Self::mul(tokens)?));
                    // println!("{:#?}", node);
                }
                TokenKind::Sub => {
                    tokens.next();
                    node = Self::new(NodeKind::Sub, Some(node), Some(Self::mul(tokens)?));
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
                    node = Self::new(NodeKind::Mul, Some(node), Some(Self::unary(tokens)?));
                }
                TokenKind::Div => {
                    tokens.next();
                    node = Self::new(NodeKind::Div, Some(node), Some(Self::unary(tokens)?));
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
                        NodeKind::Sub,
                        Some(Self::new(NodeKind::Number(0), None, None)),
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
                node = Self::new(NodeKind::Number(num), None, None);
            } else if let TokenKind::Ident = token.kind {
                let ident = token.text;
                let offset = Self::offset(ident);
                tokens.next();
                return Ok(Self::new(NodeKind::Var(offset), None, None));
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
