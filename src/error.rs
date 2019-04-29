use crate::lval::Lval;
use std::fmt;

#[derive(Debug)]
pub enum BlisprError {
    DivideByZero,
    EmptyList,
    NoChildren,
    NotANumber,
    NumArguments(usize, usize),
    ParseError(String),
    WrongType(String, String),
    UnknownFunction(String),
}

pub type BlisprResult = Result<Box<Lval>, BlisprError>;

impl fmt::Display for BlisprError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BlisprError::*;
        match self {
            DivideByZero => write!(f, "Divide by zero"),
            EmptyList => write!(f, "Empty list"),
            NoChildren => write!(f, "Lval has no children"),
            NotANumber => write!(f, "NaN"),
            NumArguments(expected, received) => write!(
                f,
                "Wrong number of arguments: expected {}, received {}",
                expected, received
            ),
            ParseError(s) => write!(f, "Parse error: {}", s),
            WrongType(expected, received) => write!(
                f,
                "Wrong type: expected {}, received {}",
                expected, received
            ),
            UnknownFunction(func_name) => write!(f, "Unknown function {}", func_name),
        }
    }
}

// unneeded?
//impl Error for BlisprError {}
