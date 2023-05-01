use crate::lval::Lval;
use std::{
	cmp::Ord,
	fmt::{self, Debug},
	hash::Hash,
	marker::Copy,
	string::ToString,
};

#[derive(Debug)]
pub enum Error {
	DivideByZero,
	EmptyList,
	FunctionFormat,
	NoChildren,
	NotANumber,
	NumArguments(usize, usize),
	Parse(String),
	Readline(String),
	WrongType(String, String),
	UnknownFunction(String),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type BlisprResult = Result<Box<Lval>>;

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Error::{
			DivideByZero, EmptyList, FunctionFormat, NoChildren, NotANumber, NumArguments, Parse,
			Readline, UnknownFunction, WrongType,
		};
		match self {
			DivideByZero => write!(f, "Divide by zero"),
			EmptyList => write!(f, "Empty list"),
			FunctionFormat => write!(
				f,
				"Function format invalid.  Symbol '&' not followed by a single symbol"
			),
			NoChildren => write!(f, "Lval has no children"),
			NotANumber => write!(f, "NaN"),
			NumArguments(expected, received) => write!(
				f,
				"Wrong number of arguments: expected {expected}, received {received}"
			),
			Parse(s) => write!(f, "Parse error: {s}"),
			Readline(s) => write!(f, "Readline error: {s}"),
			WrongType(expected, received) => write!(
				f,
				"Wrong type: expected {expected}, received {received}"
			),
			UnknownFunction(func_name) => write!(f, "Unknown function {func_name}"),
		}
	}
}

impl<T> From<pest::error::Error<T>> for Error
where
	T: Debug + Ord + Copy + Hash,
{
	fn from(error: pest::error::Error<T>) -> Self {
		Error::Parse(format!("{error}"))
	}
}

impl From<std::num::ParseIntError> for Error {
	fn from(_error: std::num::ParseIntError) -> Self {
		Error::NotANumber
	}
}

impl From<std::io::Error> for Error {
	fn from(error: std::io::Error) -> Self {
		Error::Parse(error.to_string())
	}
}

impl From<rustyline::error::ReadlineError> for Error {
	fn from(error: rustyline::error::ReadlineError) -> Self {
		Error::Readline(error.to_string())
	}
}
