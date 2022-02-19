use std::env;
// use std::error;
// use std::fmt;
use std::iter::Peekable;
use std::ops::Range;
use std::process;

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage  : ./r9cc \"<code>\"");
        eprintln!("example: ./r9cc \"4+3+10-9\"");
        process::exit(1);
    });
    let out = compile(&arg); // return Result, match and emit error message. arg is usable.
    match out {
        Ok(assemblys) => {
            println!("{}", assemblys.join("\n"));
        }
        Err(err) => {
            eprintln!("{:#?}", err);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RawStream<'a> {
    src: &'a str,
    pos: usize,
}

#[derive(Debug, PartialEq)]
struct RawTokens<'a> {
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

type Tokens<'a> = Peekable<RawTokens<'a>>;

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

#[derive(Debug, PartialEq)]
struct CompileError {
    error_type: CompileErrorType,
    pos: Range<usize>,
}

// impl fmt::Display for CompileError<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?} {:?}", self.error_type, self.pos)
//     }
// }

// impl error::Error for CompileError<'_> {}

#[derive(PartialEq, Debug)]
enum CompileErrorType {
    Tokenizing(TokenizeError),
    Parsing(ParseError),
}

#[derive(PartialEq, Debug)]
struct TokenizeError(String);

#[derive(PartialEq, Debug)]
enum ParseError {
    NotNumber,
    TrailingOp(String),
    CannotParse,
}

impl<'a> RawStream<'a> {
    fn new(src: &'a str) -> Self {
        RawStream { src, pos: 0 }
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

    fn tokenize_unknown(&mut self) -> CompileError {
        // read until space
        let (text, span) = self
            .take_while(|c| !c.is_ascii_whitespace())
            .expect("Error: whitespace only.");
        CompileError {
            error_type: CompileErrorType::Tokenizing(TokenizeError(text.to_string())),
            pos: span,
        }
    }

    // fn report_tokenize_error(&self, message: String, src_info: String) {
    //     eprintln!("{}", message);
    //     eprintln!("{}", self.src);
    //     eprintln!("{}^ {}", " ".repeat(self.pos), src_info);
    // }

    // raise tokenize error
    fn check(&mut self) -> Result<RawTokens, Vec<CompileError>> {
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
            '0'..='9' => Some(Ok(self.tokenize_number())),
            _ => {
                // self.report_tokenize_error(
                //     "[Error] Cannot tokenize".to_string(),
                //     "cannot tokenize this".to_string(),
                // );
                Some(Err(self.tokenize_unknown()))
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

    fn expr(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::mul(tokens)?;
        while let Some(token) = tokens.peek() {
            let token = token;
            match token.kind {
                TokenKind::Add => {
                    // println!("dbg! ok?");
                    tokens.next();
                    node = Self::new(TokenKind::Add, Some(node), Some(Self::mul(tokens)?));
                }
                TokenKind::Sub => {
                    tokens.next();
                    node = Self::new(TokenKind::Sub, Some(node), Some(Self::mul(tokens)?));
                }
                _ => {
                    break;
                }
            }
            // match token {
            //     Ok(token) => {

            //     },
            //     Err(err) => {
            //         return Err(err);
            //     }
            // }
        }
        Ok(node)
    }

    fn mul(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::primary(tokens)?;
        while let Some(token) = tokens.peek() {
            match token.kind {
                TokenKind::Mul => {
                    tokens.next();
                    node = Self::new(TokenKind::Mul, Some(node), Some(Self::primary(tokens)?));
                }
                TokenKind::Div => {
                    tokens.next();
                    node = Self::new(TokenKind::Div, Some(node), Some(Self::primary(tokens)?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn primary(tokens: &mut Tokens) -> Result<Node, CompileError> {
        let mut node = Self::new(TokenKind::Number(0), None, None);
        if let Some(token) = tokens.peek() {
            // println!("{:?}", token);
            if token.kind == TokenKind::Sep(Separator::RoundBracketL) {
                tokens.next();
                node = Self::expr(tokens)?;
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
        Ok(node)
    }

    fn gen(assembly: &mut Vec<String>, node: Node) {
        if let TokenKind::Number(num) = node.kind {
            let opcode = format!("\tpush {}", num);
            // println!("dbg! {}", &opcode);
            assembly.push(opcode);
            return;
        }
        if let Some(lhs) = node.lhs {
            Self::gen(assembly, *lhs);
        }
        if let Some(rhs) = node.rhs {
            Self::gen(assembly, *rhs);
        }
        assembly.push("\tpop rdi".to_string());
        assembly.push("\tpop rax".to_string());
        match node.kind {
            TokenKind::Add => {
                assembly.push("\tadd rax, rdi".to_string());
            }
            TokenKind::Sub => {
                assembly.push("\tsub rax, rdi".to_string());
            }
            TokenKind::Mul => {
                assembly.push("\timul rax, rdi".to_string());
            }
            TokenKind::Div => {
                assembly.push("\tcqo".to_string());
                assembly.push("\tidiv rdi".to_string());
            }
            _ => unreachable!(), // TODO parse error message
        }
        assembly.push("\tpush rax".to_string());
    }
}

fn compile(input: &str) -> Result<Vec<String>, Vec<CompileError>> {
    let mut tokens = RawStream::new(input);
    let mut assembly: Vec<String> = vec![".intel_syntax noprefix", ".global main", "main:"]
        .iter()
        .map(|e| e.to_string())
        .collect();
    // remove tokenize error and return tokens
    let tokens = tokens.check()?;
    let mut tokens = tokens.into_iter().peekable();
    // let mut tokens = tokens.iter().peekable();
    // println!("{:?}", tokens);
    let node = Node::expr(&mut tokens).map_err(|e| vec![e])?;
    // // println!("dbg! {:#?}", node);
    Node::gen(&mut assembly, node);
    assembly.push("\tpop rax".to_string());
    assembly.push("\tret".to_string());
    Ok(assembly)
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
fn test_error_tokenize() {
    let code = "1+22 + foo + 123 + bar";
    let mut tokens = RawStream::new(code);
    tokens.next(); // Number(1)
    tokens.next(); // Add
    tokens.next(); // Number(22)
    tokens.next(); // Add
    assert_eq!(
        tokens.next(),
        Some(Err(CompileError {
            error_type: CompileErrorType::Tokenizing(TokenizeError("foo".to_string())),
            pos: 7..10
        }))
    );
    tokens.next(); // Add
    tokens.next(); // Number(123)
    tokens.next(); // Add
    assert_eq!(
        tokens.next(),
        Some(Err(CompileError {
            error_type: CompileErrorType::Tokenizing(TokenizeError("bar".to_string())),
            pos: 19..22
        }))
    );
}

// #[test]
// fn test_cannot_tokenize() {
//     let code = compile("1+22 + foo + 123");
//     // assert_eq!(code, Err(CompileError::Tokenizing(TokenizeError)));
// }

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
