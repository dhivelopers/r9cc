use crate::errors::CompileError;
use crate::parser::Node;
use crate::tokenizer::{RawStream, TokenKind};
#[derive(Debug)]
pub struct Codegen;

impl Codegen {
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
            TokenKind::Eq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsete al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            TokenKind::NotEq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetne al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            TokenKind::Less => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetl al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            TokenKind::LessEq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetle al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            _ => unreachable!(), // TODO parse error message
        }
        assembly.push("\tpush rax".to_string());
    }

    pub fn compile(input: &str) -> Result<Vec<String>, Vec<CompileError>> {
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
        // println!("dbg! {:#?}", node);
        Self::gen(&mut assembly, node);
        assembly.push("\tpop rax".to_string());
        assembly.push("\tret".to_string());
        Ok(assembly)
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::Codegen;
    use crate::errors::{CompileError, CompileErrorType, ParseError};

    #[test]
    fn test_error_parse_not_number_1() {
        let code = "+1+22 + 123";
        let out = Codegen::compile(code);
        assert!(out.is_ok());
        assert_eq!(
            out.ok().unwrap(),
            vec![
                ".intel_syntax noprefix",
                ".global main",
                "main:",
                "\tpush 1",
                "\tpush 22",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpush 123",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpop rax",
                "\tret"
            ]
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_error_parse_not_number_2() {
        let code = "1++++22 + 123";
        let out = Codegen::compile(code);
        assert!(out.is_ok());
        assert_eq!(
            out.ok().unwrap(),
            vec![
                ".intel_syntax noprefix",
                ".global main",
                "main:",
                "\tpush 1",
                "\tpush 22",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpush 123",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpop rax",
                "\tret"
            ]
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_error_parse_trailing_1() {
        let code = "1+ 123+";
        let out = Codegen::compile(code);
        assert!(out.is_err());
        assert_eq!(
            out.err().unwrap(),
            vec![CompileError {
                error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
                pos: None,
            }]
        );
    }

    #[test]
    fn test_error_parse_trailing_2() {
        let code = "1+ 123-";
        let out = Codegen::compile(code);
        assert!(out.is_err());
        assert_eq!(
            out.err().unwrap(),
            vec![CompileError {
                error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
                pos: None,
            }]
        );
    }

    #[test]
    fn test_error_parse_not_number_3() {
        let code = "1+-22 + 123";
        let out = Codegen::compile(code);
        assert!(out.is_ok());
        assert_eq!(
            out.ok().unwrap(),
            vec![
                ".intel_syntax noprefix",
                ".global main",
                "main:",
                "\tpush 1",
                "\tpush 0",
                "\tpush 22",
                "\tpop rdi",
                "\tpop rax",
                "\tsub rax, rdi",
                "\tpush rax",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpush 123",
                "\tpop rdi",
                "\tpop rax",
                "\tadd rax, rdi",
                "\tpush rax",
                "\tpop rax",
                "\tret"
            ]
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_error_parse_cannot() {
        let code = "1 3 23";
        let out = Codegen::compile(code);
        assert!(out.is_err());
        assert_eq!(
            out.err().unwrap(),
            vec![CompileError {
                error_type: CompileErrorType::Parsing(ParseError::CannotParse),
                pos: Some(2..3),
            }]
        );
    }
}
