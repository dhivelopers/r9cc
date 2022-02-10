use std::env;
use std::ops::Range;
use std::process;

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: ./r9cc <number>");
        process::exit(1);
    });
    let code = tokenize(&arg); // TODO!
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    // println!("\tmov rax, {}", ret_value);
    println!("\tret");
}

#[derive(Debug, Clone, PartialEq)]
struct Token<'a> {
    text: &'a str,
    value: Option<usize>, // if token is Number
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
        let number: usize = text.parse().unwrap(); // text must be number, because text is collected number chars
        Token {
            text,
            value: Some(number),
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
            _ => unreachable!(),
        };
        Token {
            text: symbol,
            value: None,
            kind,
            span: start..end,
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        return match self.peek()? {
            '+' => Some(self.tokenize_reserved("+")),
            '-' => Some(self.tokenize_reserved("-")),
            '0'..='9' => Some(self.tokenize_number()),
            _ => unreachable!(),
        };
    }
}

fn tokenize(input: &str) {
    todo!()
}

#[test]
fn test_tokens_iterator() {
    let code = "5+20-4";
    let mut tokens = Tokens::new(code);
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "5",
            value: Some(5),
            kind: TokenKind::Number,
            span: 0..1
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "+",
            value: None,
            kind: TokenKind::Plus,
            span: 1..2
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "20",
            value: Some(20),
            kind: TokenKind::Number,
            span: 2..4
        })
    );
}
