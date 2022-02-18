use std::env;
use std::iter::Peekable;
use std::ops::Range;
use std::process;

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage  : ./r9cc \"<code>\"");
        eprintln!("example: ./r9cc \"4+3+10-9\"");
        process::exit(1);
    });
    compile(&arg);
}

#[derive(Debug, Clone, PartialEq)]
struct Token<'a> {
    text: &'a str,
    kind: TokenKind,
    span: Range<usize>, // Token place in Tokens
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum Separator {
    RoundBracketL, // '('
    RoundBracketR, // ')'
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TokenKind {
    Number(i64),
    Add,
    Sub,
    Mul,
    Div,
    Sep(Separator),
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
            kind: TokenKind::Number(text.parse().expect("Number parse failed.")),
            span,
        }
    }

    fn tokenize_reserved(&mut self, symbol: &'a str) -> Token<'a> {
        let start = self.pos;
        self.advance();
        let end = self.pos;
        let kind = match symbol {
            "+" => TokenKind::Add,
            "-" => TokenKind::Sub,
            "*" => TokenKind::Mul,
            "/" => TokenKind::Div,
            "(" => TokenKind::Sep(Separator::RoundBracketL),
            ")" => TokenKind::Sep(Separator::RoundBracketR),
            _ => unreachable!(), // reservedは確定しているのでunreachable
        };
        Token {
            text: symbol,
            kind,
            span: start..end,
        }
    }

    fn report_tokenize_error(&self, message: String, src_info: String) {
        eprintln!("{}", message);
        eprintln!("{}", self.src);
        eprintln!("{}^ {}", " ".repeat(self.pos), src_info);
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
            '*' => Some(self.tokenize_reserved("*")),
            '/' => Some(self.tokenize_reserved("/")),
            '(' => Some(self.tokenize_reserved("(")),
            ')' => Some(self.tokenize_reserved(")")),
            '0'..='9' => Some(self.tokenize_number()),
            _ => {
                self.report_tokenize_error(
                    "[Error] Cannot tokenize".to_string(),
                    "cannot tokenize this".to_string(),
                );
                process::exit(1);
            }
        };
    }
}

// fn report_parse_error(src: &str, span: Range<usize>, message: String, src_info: String) {
//     eprintln!("{}", message);
//     eprintln!("{}", src);
//     eprintln!(
//         "{}{} {}",
//         " ".repeat(span.start),
//         "^".repeat(span.len()),
//         src_info
//     );
// }

#[derive(Debug, Clone)]
struct Node {
    kind: TokenKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
}

impl Node {
    fn new(kind: TokenKind, lhs: Option<Node>, rhs: Option<Node>) -> Self {
        Node {
            kind,
            lhs: lhs.map(Box::new),
            rhs: rhs.map(Box::new),
        }
    }

    fn expr(tokens: &mut Peekable<Tokens>) -> Node {
        let mut node = Self::mul(tokens);
        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Add => {
                    // println!("dbg! ok?");
                    tokens.next();
                    node = Self::new(TokenKind::Add, Some(node), Some(Self::mul(tokens)));
                }
                TokenKind::Sub => {
                    tokens.next();
                    node = Self::new(TokenKind::Sub, Some(node), Some(Self::mul(tokens)));
                }
                _ => {
                    break;
                }
            }
        }
        node
    }

    fn mul(tokens: &mut Peekable<Tokens>) -> Node {
        let mut node = Self::primary(tokens);
        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Mul => {
                    tokens.next();
                    node = Self::new(TokenKind::Mul, Some(node), Some(Self::primary(tokens)));
                }
                TokenKind::Div => {
                    tokens.next();
                    node = Self::new(TokenKind::Div, Some(node), Some(Self::primary(tokens)));
                }
                _ => {
                    break;
                }
            }
        }
        node
    }

    fn primary(tokens: &mut Peekable<Tokens>) -> Node {
        let mut node = Self::new(TokenKind::Number(0), None, None);
        if let Some(token) = tokens.peek() {
            if token.kind == TokenKind::Sep(Separator::RoundBracketL) {
                tokens.next();
                node = Self::expr(tokens);
                if let Some(token) = tokens.peek() {
                    if token.kind != TokenKind::Sep(Separator::RoundBracketR) {
                        process::exit(1); // TODO parse error message
                    } else {
                        tokens.next();
                    }
                } else {
                    process::exit(1); // TODO parse error message
                }
            } else if let TokenKind::Number(num) = token.kind {
                tokens.next();
                node = Self::new(TokenKind::Number(num), None, None);
            }
        }
        node
    }

    fn gen(node: Node) {
        if let TokenKind::Number(num) = node.kind {
            println!("\tpush {num}");
            return;
        }
        if let Some(lhs) = node.lhs {
            Self::gen(*lhs);
        }
        if let Some(rhs) = node.rhs {
            Self::gen(*rhs);
        }
        println!("\tpop rdi");
        println!("\tpop rax");
        match node.kind {
            TokenKind::Add => {
                println!("\tadd rax, rdi");
            }
            TokenKind::Sub => {
                println!("\tsub rax, rdi");
            }
            TokenKind::Mul => {
                println!("\timul rax, rdi");
            }
            TokenKind::Div => {
                println!("\tcqo");
                println!("\tidiv rdi");
            }
            _ => unreachable!(), // TODO parse error message
        }
        println!("\tpush rax");
    }
}

fn compile(input: &str) {
    let tokens = Tokens::new(input);
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    let mut tokens = tokens.peekable();
    let node = Node::expr(&mut tokens);
    // println!("dbg! {:#?}", node);
    Node::gen(node);
    println!("\tpop rax");
    println!("\tret");
}

#[test]
fn test_tokens_iterator() {
    let code = "5+20-4";
    let mut tokens = Tokens::new(code);
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "5",
            kind: TokenKind::Number(5),
            span: 0..1
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "+",
            kind: TokenKind::Add,
            span: 1..2
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "20",
            kind: TokenKind::Number(20),
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
            kind: TokenKind::Number(3),
            span: 2..3
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "-",
            kind: TokenKind::Sub,
            span: 5..6
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "1",
            kind: TokenKind::Number(1),
            span: 6..7
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "+",
            kind: TokenKind::Add,
            span: 9..10
        })
    );
    assert_eq!(
        tokens.next(),
        Some(Token {
            text: "20",
            kind: TokenKind::Number(20),
            span: 10..12
        })
    );
    assert_eq!(tokens.next(), None);
}

// TODO error test
// test case
/*
1+22 + foo + 123 => cannot tokenize this
+1+22 + foo + 123 => This is not `Number`
1++++22 + foo + 123 => This is not `Number`
1+ 123+ => This is end of source code, add `Number` after `+` if needed
1+ 123- => This is end of source code, add `Number` after `-` if needed
1+-22 + foo + 123 => This is not `Number`
1 3 23 => cannot parse this
*/
