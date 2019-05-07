use crate::lval::Lval;
use std::{
    cmp::Ord,
    fmt::{self, Debug},
    hash::Hash,
    marker::Copy,
    string::ToString,
};

#[derive(Debug)]
pub enum BlisprError {
    DivideByZero,
    EmptyList,
    NoChildren,
    NotANumber,
    NumArguments(usize, usize),
    ParseError(String),
    ReadlineError(String),
    WrongType(String, String),
    UnknownFunction(String),
}

pub type Result<T> = std::result::Result<T, BlisprError>;
pub type BlisprResult = Result<Box<Lval>>;

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
            ReadlineError(s) => write!(f, "Readline error: {}", s),
            WrongType(expected, received) => write!(
                f,
                "Wrong type: expected {}, received {}",
                expected, received
            ),
            UnknownFunction(func_name) => write!(f, "Unknown function {}", func_name),
        }
    }
}

impl<T> From<pest::error::Error<T>> for BlisprError
where
    T: Debug + Ord + Copy + Hash,
{
    fn from(error: pest::error::Error<T>) -> Self {
        BlisprError::ParseError(format!("{}", error))
    }
}

impl From<std::num::ParseIntError> for BlisprError {
    fn from(_error: std::num::ParseIntError) -> Self {
        BlisprError::NotANumber
    }
}

impl From<std::io::Error> for BlisprError {
    fn from(error: std::io::Error) -> Self {
        BlisprError::ParseError(error.to_string())
    }
}

impl From<rustyline::error::ReadlineError> for BlisprError {
    fn from(error: rustyline::error::ReadlineError) -> Self {
        BlisprError::ReadlineError(error.to_string())
    }
}
