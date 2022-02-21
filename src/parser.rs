use std::collections::HashMap;

use crate::errors::{CompileError, CompileErrorType, ParseError};
use crate::tokenizer::{Separator, TokenKind, Tokens};

#[derive(Debug)]
pub struct LocalVar {
    offset: usize,
}

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
    Return, // 'return'
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct Parser {
    locals: HashMap<String, LocalVar>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            locals: HashMap::new(),
        }
    }

    fn offset(&self) -> usize {
        (self.locals.len() + 1) * 8
    }

    pub fn program(&mut self, tokens: &mut Tokens) -> Result<Vec<Node>, CompileError> {
        let mut code = vec![];
        while tokens.peek().is_some() {
            code.push(self.stmt(tokens)?);
        }
        Ok(code)
    }

    fn stmt(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let node;
        if let Some(token) = tokens.peek() {
            if token.kind == TokenKind::Return {
                tokens.next();
                node = Node::new(NodeKind::Return, Some(self.expr(tokens)?), None);
            } else {
                node = self.expr(tokens)?;
            }
        } else {
            return Err(CompileError {
                error_type: CompileErrorType::Parsing(ParseError::Empty),
                pos: None,
            });
        }
        if let Some(token) = tokens.peek() {
            if token.kind != TokenKind::Sep(Separator::SemiColon) {
                return Err(CompileError {
                    error_type: CompileErrorType::Parsing(ParseError::NeedSemiColon),
                    pos: Some(token.span.clone()),
                });
            } else {
                tokens.next(); // eat ';'
            }
        } else {
            return Err(CompileError {
                error_type: CompileErrorType::Parsing(ParseError::NeedSemiColon),
                pos: None,
            });
        }
        Ok(node)
    }

    fn expr(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        self.assign(tokens)
    }

    fn assign(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = self.equality(tokens)?;
        if let Some(token) = tokens.peek() {
            if token.kind == TokenKind::Assign {
                tokens.next();
                node = Node::new(NodeKind::Assign, Some(node), Some(self.assign(tokens)?));
            }
        }
        Ok(node)
    }

    fn equality(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = self.relational(tokens)?;

        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Eq => {
                    tokens.next();
                    node = Node::new(NodeKind::Eq, Some(node), Some(self.relational(tokens)?));
                }
                TokenKind::NotEq => {
                    tokens.next();
                    node = Node::new(NodeKind::NotEq, Some(node), Some(self.relational(tokens)?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn relational(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = self.add(tokens)?;
        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Less => {
                    tokens.next();
                    node = Node::new(NodeKind::Less, Some(node), Some(self.add(tokens)?));
                }
                TokenKind::LessEq => {
                    tokens.next();
                    node = Node::new(NodeKind::LessEq, Some(node), Some(self.add(tokens)?));
                }
                TokenKind::Greater => {
                    tokens.next();
                    node = Node::new(NodeKind::Less, Some(self.add(tokens)?), Some(node));
                }
                TokenKind::GreaterEq => {
                    tokens.next();
                    node = Node::new(NodeKind::LessEq, Some(self.add(tokens)?), Some(node));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn add(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        // println!("{:#?}", tokens);
        let mut node = self.mul(tokens)?;
        while let Some(token) = tokens.peek() {
            // println!("expr: {:#?}", token);
            match token.kind {
                TokenKind::Add => {
                    // println!("dbg! ok?");
                    tokens.next();
                    node = Node::new(NodeKind::Add, Some(node), Some(self.mul(tokens)?));
                    // println!("{:#?}", node);
                }
                TokenKind::Sub => {
                    tokens.next();
                    node = Node::new(NodeKind::Sub, Some(node), Some(self.mul(tokens)?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn mul(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = self.unary(tokens)?;
        while let Some(token) = tokens.peek() {
            // println!("mul: {:#?}", token);
            match token.kind {
                TokenKind::Mul => {
                    tokens.next();
                    node = Node::new(NodeKind::Mul, Some(node), Some(self.unary(tokens)?));
                }
                TokenKind::Div => {
                    tokens.next();
                    node = Node::new(NodeKind::Div, Some(node), Some(self.unary(tokens)?));
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

    fn unary(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let result;
        // println!("{:#?}", result);
        if let Some(token) = tokens.peek() {
            // println!("unary: {:#?}", token);
            match token.kind {
                TokenKind::Add => {
                    // println!("+ {:?}", token.span);
                    tokens.next();
                    result = self.unary(tokens);
                }
                TokenKind::Sub => {
                    tokens.next();
                    result = Ok(Node::new(
                        NodeKind::Sub,
                        Some(Node::new(NodeKind::Number(0), None, None)),
                        Some(self.unary(tokens)?),
                    ));
                }
                _ => {
                    result = self.primary(tokens);
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

    fn primary(&mut self, tokens: &mut Tokens) -> Result<Node, CompileError> {
        let node;
        if let Some(token) = tokens.peek() {
            // println!("primary: {:#?}", token);
            let span = &token.span;
            // println!("{:?}", token);
            if token.kind == TokenKind::Sep(Separator::RoundBracketL) {
                tokens.next();
                node = self.expr(tokens)?;
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
                node = Node::new(NodeKind::Number(num), None, None);
            } else if let TokenKind::Ident = token.kind {
                // Convert `ident` -> `var`
                let ident = token.text;
                // Search offset by ident name
                #[allow(clippy::map_entry)]
                let offset = if !self.locals.contains_key(&ident.to_string()) {
                    let offset = self.offset();
                    self.locals.insert(ident.to_string(), LocalVar { offset });
                    offset
                } else {
                    self.locals[&ident.to_string()].offset
                };
                tokens.next();
                return Ok(Node::new(NodeKind::Var(offset), None, None));
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
