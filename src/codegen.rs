use crate::errors::{CodegenError, CompileError, CompileErrorType};
use crate::parser::{Node, NodeKind, Parser};
use crate::tokenizer::RawStream;

#[derive(Debug)]
pub struct Codegen;

impl Codegen {
    fn gen(assembly: &mut Vec<String>, nodes: Vec<Node>) -> Result<(), CompileError> {
        for node in nodes {
            Self::gen_code(assembly, node)?;
            assembly.push("\tpop rax".to_string());
        }
        Ok(())
    }
    fn gen_code(assembly: &mut Vec<String>, node: Node) -> Result<(), CompileError> {
        match node.kind {
            NodeKind::Return => {
                if let Some(lhs) = node.lhs {
                    Self::gen_code(assembly, *lhs)?;
                }
                assembly.push("\tpop rax".to_string());
                assembly.push("\tmov rsp, rbp".to_string());
                assembly.push("\tpop rbp".to_string());
                assembly.push("\tret".to_string());
                return Ok(());
            }
            NodeKind::Number(num) => {
                let opcode = format!("\tpush {}", num);
                // println!("dbg! {}", &opcode);
                assembly.push(opcode);
                return Ok(());
            }
            NodeKind::Var(val) => {
                assembly.push("\tmov rax, rbp".to_string());
                assembly.push(format!("\tsub rax, {}", val));
                assembly.push("\tpush rax".to_string());
                assembly.push("\tpop rax".to_string());
                assembly.push("\tmov rax, [rax]".to_string());
                assembly.push("\tpush rax".to_string());
                return Ok(());
            }
            NodeKind::Assign => {
                if let Some(lhs) = node.lhs {
                    let node = *lhs;
                    if let NodeKind::Var(val) = node.kind {
                        assembly.push("\tmov rax, rbp".to_string());
                        assembly.push(format!("\tsub rax, {}", val));
                        assembly.push("\tpush rax".to_string());
                    } else {
                        return Err(CompileError {
                            error_type: CompileErrorType::Codegen(CodegenError::LValueNotVar),
                            pos: None,
                        });
                    }
                } else {
                    return Err(CompileError {
                        error_type: CompileErrorType::Codegen(CodegenError::LValueNotVar),
                        pos: None,
                    });
                }
                if let Some(rhs) = node.rhs {
                    Self::gen_code(assembly, *rhs)?;
                } else {
                    return Err(CompileError {
                        error_type: CompileErrorType::Codegen(CodegenError::RValueNotFound),
                        pos: None,
                    });
                }
                // 先にスタックに積んだ値がlvalueなのでraxにpopする
                // 次にスタックに積まれた値はrvalueなのでrdiにpopする
                assembly.push("\tpop rdi".to_string());
                assembly.push("\tpop rax".to_string());
                assembly.push("\tmov [rax], rdi".to_string());
                assembly.push("\tpush rdi".to_string());
                return Ok(());
            }
            _ => {}
        }
        if let Some(lhs) = node.lhs {
            Self::gen_code(assembly, *lhs)?;
        }
        if let Some(rhs) = node.rhs {
            Self::gen_code(assembly, *rhs)?;
        }
        assembly.push("\tpop rdi".to_string());
        assembly.push("\tpop rax".to_string());
        match node.kind {
            NodeKind::Add => {
                assembly.push("\tadd rax, rdi".to_string());
            }
            NodeKind::Sub => {
                assembly.push("\tsub rax, rdi".to_string());
            }
            NodeKind::Mul => {
                assembly.push("\timul rax, rdi".to_string());
            }
            NodeKind::Div => {
                assembly.push("\tcqo".to_string());
                assembly.push("\tidiv rdi".to_string());
            }
            NodeKind::Eq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsete al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            NodeKind::NotEq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetne al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            NodeKind::Less => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetl al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            NodeKind::LessEq => {
                assembly.push("\tcmp rax, rdi".to_string());
                assembly.push("\tsetle al".to_string());
                assembly.push("\tmovzb rax, al".to_string());
            }
            _ => unreachable!(), // TODO parse error message
        }
        assembly.push("\tpush rax".to_string());
        Ok(())
    }

    pub fn compile(input: &str) -> Result<Vec<String>, Vec<CompileError>> {
        let mut tokens = RawStream::new(input);
        let mut assembly: Vec<String> = vec![
            ".intel_syntax noprefix",
            ".global main",
            "main:",
            "\tpush rbp",
            "\tmov rbp, rsp",
            "\tsub rsp, 208",
        ]
        .iter()
        .map(|e| e.to_string())
        .collect();
        // remove tokenize error and return tokens
        let tokens = tokens.check()?;
        let mut tokens = tokens.into_iter().peekable();
        // let mut tokens = tokens.iter().peekable();
        // println!("{:?}", tokens);
        let mut parser = Parser::new();
        let node = parser.program(&mut tokens).map_err(|e| vec![e])?;
        // println!("dbg! {:#?}", node);
        Self::gen(&mut assembly, node).map_err(|e| vec![e])?;
        assembly.push("\tmov rsp, rbp".to_string());
        assembly.push("\tpop rbp".to_string());
        assembly.push("\tret".to_string());
        Ok(assembly)
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::codegen::Codegen;
//     use crate::errors::{CompileError, CompileErrorType, ParseError};

//     #[test]
//     fn test_error_parse_not_number_1() {
//         let code = "+1+22 + 123;";
//         let out = Codegen::compile(code);
//         assert!(out.is_ok());
//         assert_eq!(
//             out.ok().unwrap(),
//             vec![
//                 ".intel_syntax noprefix",
//                 ".global main",
//                 "main:",
//                 "\tpush rbp",
//                 "\tmov rbp, rsp",
//                 "\tsub rsp, 208",
//                 "\tpush 1",
//                 "\tpush 22",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpush 123",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpop rax",
//                 "\tmov rsp, rbp",
//                 "\tpop rbp",
//                 "\tret"
//             ]
//             .iter()
//             .map(|e| e.to_string())
//             .collect::<Vec<String>>()
//         );
//     }

//     #[test]
//     fn test_error_parse_not_number_2() {
//         let code = "1++++22 + 123;";
//         let out = Codegen::compile(code);
//         assert!(out.is_ok());
//         assert_eq!(
//             out.ok().unwrap(),
//             vec![
//                 ".intel_syntax noprefix",
//                 ".global main",
//                 "main:",
//                 "\tpush rbp",
//                 "\tmov rbp, rsp",
//                 "\tsub rsp, 208",
//                 "\tpush 1",
//                 "\tpush 22",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpush 123",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpop rax",
//                 "\tmov rsp, rbp",
//                 "\tpop rbp",
//                 "\tret"
//             ]
//             .iter()
//             .map(|e| e.to_string())
//             .collect::<Vec<String>>()
//         );
//     }

//     #[test]
//     fn test_error_parse_trailing_1() {
//         let code = "1+ 123+";
//         let out = Codegen::compile(code);
//         assert!(out.is_err());
//         assert_eq!(
//             out.err().unwrap(),
//             vec![CompileError {
//                 error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
//                 pos: None,
//             }]
//         );
//     }

//     #[test]
//     fn test_error_parse_trailing_2() {
//         let code = "1+ 123-";
//         let out = Codegen::compile(code);
//         assert!(out.is_err());
//         assert_eq!(
//             out.err().unwrap(),
//             vec![CompileError {
//                 error_type: CompileErrorType::Parsing(ParseError::TrailingOp),
//                 pos: None,
//             }]
//         );
//     }

//     #[test]
//     fn test_error_parse_not_number_3() {
//         let code = "1+-22 + 123;";
//         let out = Codegen::compile(code);
//         assert!(out.is_ok());
//         assert_eq!(
//             out.ok().unwrap(),
//             vec![
//                 ".intel_syntax noprefix",
//                 ".global main",
//                 "main:",
//                 "\tpush rbp",
//                 "\tmov rbp, rsp",
//                 "\tsub rsp, 208",
//                 "\tpush 1",
//                 "\tpush 0",
//                 "\tpush 22",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tsub rax, rdi",
//                 "\tpush rax",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpush 123",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tadd rax, rdi",
//                 "\tpush rax",
//                 "\tpop rax",
//                 "\tmov rsp, rbp",
//                 "\tpop rbp",
//                 "\tret"
//             ]
//             .iter()
//             .map(|e| e.to_string())
//             .collect::<Vec<String>>()
//         );
//     }

//     #[test]
//     fn test_error_parse_cannot() {
//         let code = "1 3 23;";
//         let out = Codegen::compile(code);
//         assert!(out.is_err());
//         assert_eq!(
//             out.err().unwrap(),
//             vec![CompileError {
//                 error_type: CompileErrorType::Parsing(ParseError::CannotParse),
//                 pos: Some(2..3),
//             }]
//         );
//     }

//     #[test]
//     fn test_error_no_semicolon() {
//         let code = "1 + 2 ";
//         let out = Codegen::compile(code);
//         assert!(out.is_err());
//         assert_eq!(
//             out.err().unwrap(),
//             vec![CompileError {
//                 error_type: CompileErrorType::Parsing(ParseError::NeedSemiColon),
//                 pos: None,
//             }]
//         );
//     }

//     #[test]
//     fn test_variable() {
//         let code = "a=3;a;";
//         let out = Codegen::compile(code);
//         assert!(out.is_ok());
//         assert_eq!(
//             out.ok().unwrap(),
//             vec![
//                 ".intel_syntax noprefix",
//                 ".global main",
//                 "main:",
//                 "\tpush rbp",
//                 "\tmov rbp, rsp",
//                 "\tsub rsp, 208",
//                 "\tmov rax, rbp",
//                 "\tsub rax, 8",
//                 "\tpush rax",
//                 "\tpush 3",
//                 "\tpop rdi",
//                 "\tpop rax",
//                 "\tmov [rax], rdi",
//                 "\tpush rdi",
//                 "\tpop rax",
//                 "\tmov rax, rbp",
//                 "\tsub rax, 8",
//                 "\tpush rax",
//                 "\tpop rax",
//                 "\tmov rax, [rax]",
//                 "\tpush rax",
//                 "\tpop rax",
//                 "\tmov rsp, rbp",
//                 "\tpop rbp",
//                 "\tret"
//             ]
//             .iter()
//             .map(|e| e.to_string())
//             .collect::<Vec<String>>()
//         );
//     }
// }
