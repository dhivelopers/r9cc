use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct CompileError {
    pub error_type: CompileErrorType,
    pub pos: Option<Range<usize>>,
}

// impl fmt::Display for CompileError<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?} {:?}", self.error_type, self.pos)
//     }
// }

// impl error::Error for CompileError<'_> {}

#[derive(PartialEq, Debug)]
pub enum CompileErrorType {
    Tokenizing(TokenizeError),
    Parsing(ParseError),
    Codegen(CodegenError),
}

#[derive(PartialEq, Debug)]
pub struct TokenizeError(pub String);

#[derive(PartialEq, Debug)]
pub enum ParseError {
    NotNumber,
    TrailingOp,
    CannotParse,
    NotFoundRoundBracketR,
    NeedSemiColon,
}

#[derive(PartialEq, Debug)]
pub enum CodegenError {
    LValueNotVar,   // left value is not variable
    RValueNotFound, // assign error
}
