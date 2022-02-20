use crate::errors::{CompileError, CompileErrorType, TokenizeError};
use std::iter::Peekable;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct RawStream<'a> {
    src: &'a str,
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub struct RawTokens<'a> {
    raw_tokens: Vec<Token<'a>>,
    index: usize,
}

impl<'a> Iterator for RawTokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index > self.raw_tokens.len() {
            None
        } else {
            Some(self.raw_tokens[self.index - 1].clone())
        }
    }
}

pub type Tokens<'a> = Peekable<RawTokens<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
    pub span: Range<usize>, // Token place in Tokens
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Separator {
    RoundBracketL, // '('
    RoundBracketR, // ')'
    SemiColon,     // ';'
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
    Ident, // identifier
    Number(i64),
    Add,
    Sub,
    Mul,
    Div,
    Eq,        // '=='
    NotEq,     // '!='
    Less,      // '<'
    LessEq,    // '<='
    Greater,   // '>'
    GreaterEq, // '>='
    Assign,    // '='
    Sep(Separator),
}

impl<'a> RawStream<'a> {
    pub fn new(src: &'a str) -> Self {
        RawStream { src, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek2(&self) -> (Option<char>, Option<char>) {
        (self.rest().chars().next(), self.rest().chars().nth(1))
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn take_while<T>(&mut self, mut predicate: T) -> Option<(&'a str, Range<usize>)>
    where
        T: FnMut(char) -> bool,
    {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            }
            self.advance();
        }

        let end = self.pos;

        if start != end {
            let text = &self.src[start..end];
            Some((text, start..end))
        } else {
            None
        }
    }
    fn tokenize_number(&mut self) -> Token<'a> {
        let (text, span) = self
            .take_while(|c| matches!(c, '0'..='9'))
            .expect("Error: No digit.");
        Token {
            text,
            kind: TokenKind::Number(text.parse().expect("Number parse failed.")),
            span,
        }
    }

    fn tokenize_reserved(&mut self, symbol: &'a str) -> Token<'a> {
        let start = self.pos;
        for _ in 0..symbol.len() {
            self.advance();
        }
        let end = self.pos;
        let kind = match symbol {
            "+" => TokenKind::Add,
            "-" => TokenKind::Sub,
            "*" => TokenKind::Mul,
            "/" => TokenKind::Div,
            "(" => TokenKind::Sep(Separator::RoundBracketL),
            ")" => TokenKind::Sep(Separator::RoundBracketR),
            ";" => TokenKind::Sep(Separator::SemiColon),
            "==" => TokenKind::Eq,
            "!=" => TokenKind::NotEq,
            "<" => TokenKind::Less,
            "<=" => TokenKind::LessEq,
            ">" => TokenKind::Greater,
            ">=" => TokenKind::GreaterEq,
            "=" => TokenKind::Assign,
            _ => unreachable!(), // reservedは確定しているのでunreachable
        };
        Token {
            text: symbol,
            kind,
            span: start..end,
        }
    }

    fn tokenize_identifier(&mut self) -> Token<'a> {
        let start = self.pos;
        self.advance();
        let end = self.pos;
        let ident = &self.src[start..end];
        Token {
            text: ident,
            kind: TokenKind::Ident,
            span: start..end,
        }
    }

    fn tokenize_unknown(&mut self) -> CompileError {
        // read until space
        let (text, span) = self
            .take_while(|c| !c.is_ascii_whitespace())
            .expect("Error: whitespace only.");
        CompileError {
            error_type: CompileErrorType::Tokenizing(TokenizeError(text.to_string())),
            pos: Some(span),
        }
    }

    // raise tokenize error
    pub fn check(&mut self) -> Result<RawTokens, Vec<CompileError>> {
        let (tokens, errors): (Vec<_>, Vec<_>) = self.into_iter().partition(Result::is_ok);
        let tokens: Vec<Token> = tokens.into_iter().map(Result::unwrap).collect();
        let errors: Vec<CompileError> = errors.into_iter().map(Result::unwrap_err).collect();
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(RawTokens {
                raw_tokens: tokens,
                index: 0,
            })
        }
    }
}

impl<'a> Iterator for RawStream<'a> {
    type Item = Result<Token<'a>, CompileError>;

    fn next(&mut self) -> Option<Self::Item> {
        // ignore spaces
        while let Some(x) = self.peek() {
            if x.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        return match self.peek()? {
            '+' => Some(Ok(self.tokenize_reserved("+"))),
            '-' => Some(Ok(self.tokenize_reserved("-"))),
            '*' => Some(Ok(self.tokenize_reserved("*"))),
            '/' => Some(Ok(self.tokenize_reserved("/"))),
            '(' => Some(Ok(self.tokenize_reserved("("))),
            ')' => Some(Ok(self.tokenize_reserved(")"))),
            ';' => Some(Ok(self.tokenize_reserved(";"))),
            '0'..='9' => Some(Ok(self.tokenize_number())),
            'a'..='z' => Some(Ok(self.tokenize_identifier())),
            _ => match self.peek2() {
                (Some('='), Some('=')) => Some(Ok(self.tokenize_reserved("=="))),
                (Some('!'), Some('=')) => Some(Ok(self.tokenize_reserved("!="))),
                (Some('<'), Some('=')) => Some(Ok(self.tokenize_reserved("<="))),
                (Some('>'), Some('=')) => Some(Ok(self.tokenize_reserved(">="))),
                (Some('<'), _) => Some(Ok(self.tokenize_reserved("<"))),
                (Some('>'), _) => Some(Ok(self.tokenize_reserved(">"))),
                (Some('='), _) => Some(Ok(self.tokenize_reserved("="))),
                _ => Some(Err(self.tokenize_unknown())),
            },
        };
    }
}

#[test]
fn test_tokens_iterator() {
    let code = "5+20-4";
    let mut tokens = RawStream::new(code);
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "5",
            kind: TokenKind::Number(5),
            span: 0..1
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "+",
            kind: TokenKind::Add,
            span: 1..2
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "20",
            kind: TokenKind::Number(20),
            span: 2..4
        }))
    );
}

#[test]
fn test_whitespace() {
    let code = "  3  -1  +20  ";
    let mut tokens = RawStream::new(code);
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "3",
            kind: TokenKind::Number(3),
            span: 2..3
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "-",
            kind: TokenKind::Sub,
            span: 5..6
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "1",
            kind: TokenKind::Number(1),
            span: 6..7
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "+",
            kind: TokenKind::Add,
            span: 9..10
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "20",
            kind: TokenKind::Number(20),
            span: 10..12
        }))
    );
    assert_eq!(tokens.next(), None);
}

#[test]
fn test_ident() {
    let code = "a + b - c";
    let mut tokens = RawStream::new(code);
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "a",
            kind: TokenKind::Ident,
            span: 0..1
        }))
    );
    tokens.next(); // Add
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "b",
            kind: TokenKind::Ident,
            span: 4..5
        }))
    );
    tokens.next(); // Sub
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "c",
            kind: TokenKind::Ident,
            span: 8..9
        }))
    );
}

#[test]
fn test_semicolon() {
    let code = "a + b; c";
    let mut tokens = RawStream::new(code);
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "a",
            kind: TokenKind::Ident,
            span: 0..1
        }))
    );
    tokens.next(); // Add
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: "b",
            kind: TokenKind::Ident,
            span: 4..5
        }))
    );
    assert_eq!(
        tokens.next(),
        Some(Ok(Token {
            text: ";",
            kind: TokenKind::Sep(Separator::SemiColon),
            span: 5..6
        }))
    );
}

// #[test]
// fn test_error_tokenize() {
//     let code = "1+22 + foo + 123 + bar";
//     let mut tokens = RawStream::new(code);
//     tokens.next(); // Number(1)
//     tokens.next(); // Add
//     tokens.next(); // Number(22)
//     tokens.next(); // Add
//     assert_eq!(
//         tokens.next(),
//         Some(Err(CompileError {
//             error_type: CompileErrorType::Tokenizing(TokenizeError("foo".to_string())),
//             pos: Some(7..10)
//         }))
//     );
//     tokens.next(); // Add
//     tokens.next(); // Number(123)
//     tokens.next(); // Add
//     assert_eq!(
//         tokens.next(),
//         Some(Err(CompileError {
//             error_type: CompileErrorType::Tokenizing(TokenizeError("bar".to_string())),
//             pos: Some(19..22)
//         }))
//     );
// }
