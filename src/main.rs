use std::env;
use std::ops::Range;
use std::process;

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage  : ./r9cc \"<code>\"");
        eprintln!("example: ./r9cc \"4+3+10-9\"");
        process::exit(1);
    });
    let code = compile(&arg);
    println!("{code}");
}

#[derive(Debug, Clone, PartialEq)]
struct Token<'a> {
    text: &'a str,
    kind: TokenKind,
    span: Range<usize>, // Token place in Tokens
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TokenKind {
    Number,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
struct Tokens<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Tokens<'a> {
    fn new(src: &'a str) -> Self {
        Tokens { src, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
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
            kind: TokenKind::Number,
            span,
        }
    }

    fn tokenize_reserved(&mut self, symbol: &'a str) -> Token<'a> {
        let start = self.pos;
        self.advance();
        let end = self.pos;
        let kind = match symbol {
            "+" => TokenKind::Plus,
            "-" => TokenKind::Minus,
            _ => unreachable!(), // reservedは確定しているのでunreachable
        };
        Token {
            text: symbol,
            kind,
            span: start..end,
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

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
            '+' => Some(self.tokenize_reserved("+")),
            '-' => Some(self.tokenize_reserved("-")),
            '0'..='9' => Some(self.tokenize_number()),
            _ => unreachable!(), // TODO tokenize error message
        };
    }
}

fn compile(input: &str) -> String {
    let mut tokens = Tokens::new(input);
    let mut assembly: Vec<String> = vec![
        ".intel_syntax noprefix".to_string(),
        ".global main".to_string(),
        "main:".to_string(),
    ];
    if let Some(first_token) = tokens.next() {
        // first_token must be Number.
        if first_token.kind != TokenKind::Number {
            println!("first_token must be Number."); // TODO parse error message
            process::exit(1);
        }
        assembly.push(format!("\tmov rax, {}", first_token.text));
    }
    while let Some(token) = tokens.next() {
        match token.kind {
            TokenKind::Plus => {
                if let Some(num_tok) = tokens.next() {
                    if num_tok.kind == TokenKind::Number {
                        assembly.push(format!("\tadd rax, {}", num_tok.text));
                    } else {
                        println!("+<number>"); // TODO parse error message
                    }
                } else {
                    println!("+<something>"); // TODO parse error message
                }
            }
            TokenKind::Minus => {
                if let Some(num_tok) = tokens.next() {
                    if num_tok.kind == TokenKind::Number {
                        assembly.push(format!("\tsub rax, {}", num_tok.text));
                    } else {
                        println!("-<number>"); // TODO parse error message
                    }
                } else {
                    println!("-<something>"); // TODO parse error message
                }
            }
            _ => unreachable!(), // TODO parse error message
        }
    }
    assembly.push("\tret".to_string());
    assembly.join("\n")
}

#[test]
fn test_tokens_iterator() {
    let code = "5+20-4";
    let mut tokens = Tokens::new(code);
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "5",
            kind: TokenKind::Number,
            span: 0..1
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "+",
            kind: TokenKind::Plus,
            span: 1..2
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "20",
            kind: TokenKind::Number,
            span: 2..4
        })
    );
}

#[test]
fn test_whitespace() {
    let code = "  3  -1  +20  ";
    let mut tokens = Tokens::new(code);
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "3",
            kind: TokenKind::Number,
            span: 2..3
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "-",
            kind: TokenKind::Minus,
            span: 5..6
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "1",
            kind: TokenKind::Number,
            span: 6..7
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "+",
            kind: TokenKind::Plus,
            span: 9..10
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "20",
            kind: TokenKind::Number,
            span: 10..12
        })
    );
    assert_eq!(tokens.next(), None);
}
