use crate::error::{BlisprResult, Error, Result};
use std::{collections::HashMap, fmt};

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren = Vec<Box<Lval>>;
pub type LBuiltin = fn(&mut Lval) -> BlisprResult;

// There are two types of function - builtin and lambda
#[derive(Clone)]
pub enum Func {
	Builtin(String, LBuiltin), // (name, function pointer)
	Lambda(HashMap<String, Box<Lval>>, Box<Lval>, Box<Lval>), // (environment(?), formals, body), both should be Qexpr // TODO these should both be Rc<T>
}

// The book has a pointer to an Lenv in the Lambda
// I instead just store a plain old hashmap of any extras
// it's then applied in lval_call

// The main type - all possible Blispr values
#[derive(Debug, Clone, PartialEq)]
pub enum Lval {
	Blispr(LvalChildren),
	Fun(Func),
	Num(i64),
	Sym(String),
	Sexpr(LvalChildren),
	Qexpr(LvalChildren),
}

impl Lval {
	pub fn as_num(&self) -> Result<i64> {
		match *self {
			Lval::Num(n_num) => Ok(n_num),
			_ => Err(Error::NotANumber),
		}
	}
	pub fn as_string(&self) -> Result<String> {
		match self {
			Lval::Sym(s) => Ok(s.to_string()),
			_ => Err(Error::WrongType("symbol".to_string(), format!("{}", self))),
		}
	}
	pub fn len(&self) -> Result<usize> {
		match *self {
			Lval::Sexpr(ref children) | Lval::Qexpr(ref children) | Lval::Blispr(ref children) => {
				Ok(children.len())
			},
			_ => Err(Error::NoChildren),
		}
	}
}

impl fmt::Debug for Func {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Func::Builtin(name, _) => write!(f, "Builtin({})", name),
			Func::Lambda(env, formals, body) => {
				write!(f, "Lambda({{{:?}}},{{{}}},{{{}}})", env, formals, body)
			},
		}
	}
}

impl PartialEq for Func {
	fn eq(&self, other: &Func) -> bool {
		match self {
			Func::Builtin(name, _) => match other {
				Func::Builtin(other_name, _) => name == other_name,
				Func::Lambda(..) => false,
			},
			Func::Lambda(env, formals, body) => match other {
				Func::Lambda(other_env, other_f, other_b) => {
					formals == other_f && body == other_b && env == other_env
				},
				Func::Builtin(..) => false,
			},
		}
	}
}

impl fmt::Display for Lval {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Lval::Blispr(_cells) => write!(f, "<toplevel>"),
			Lval::Fun(lf) => match lf {
				Func::Builtin(name, _) => write!(f, "<builtin: {}>", name),
				Func::Lambda(_, formals, body) => write!(f, "(\\ {} {})", formals, body),
			},
			Lval::Num(n) => write!(f, "{}", n),
			Lval::Sym(s) => write!(f, "{}", s),
			Lval::Sexpr(cell) => write!(f, "({})", lval_expr_print(cell)),
			Lval::Qexpr(cell) => write!(f, "{{{}}}", lval_expr_print(cell)),
		}
	}
}

fn lval_expr_print(cell: &[Box<Lval>]) -> String {
	let mut ret = String::new();
	for i in 0..cell.len() {
		ret.push_str(&format!("{}", cell[i]));
		if i < cell.len() - 1 {
			ret.push(' ');
		}
	}
	ret
}

// Constructors
// Each allocates a brand new boxed Lval
// The recursive types start empty

pub fn blispr() -> Box<Lval> {
	Box::new(Lval::Blispr(Vec::new()))
}

pub fn builtin(f: LBuiltin, name: &str) -> Box<Lval> {
	Box::new(Lval::Fun(Func::Builtin(name.to_string(), f)))
}

pub fn lambda(env: HashMap<String, Box<Lval>>, formals: Box<Lval>, body: Box<Lval>) -> Box<Lval> {
	Box::new(Lval::Fun(Func::Lambda(env, formals, body)))
}

pub fn num(n: i64) -> Box<Lval> {
	Box::new(Lval::Num(n))
}

pub fn sym(s: &str) -> Box<Lval> {
	Box::new(Lval::Sym(s.into()))
}

pub fn sexpr() -> Box<Lval> {
	Box::new(Lval::Sexpr(Vec::new()))
}

pub fn qexpr() -> Box<Lval> {
	Box::new(Lval::Qexpr(Vec::new()))
}

// Manipulating children

// Add lval x to lval::sexpr or lval::qexpr v
pub fn add(v: &mut Lval, x: &Lval) -> Result<()> {
	match *v {
		Lval::Sexpr(ref mut children)
		| Lval::Qexpr(ref mut children)
		| Lval::Blispr(ref mut children) => {
			children.push(Box::new(x.clone()));
		},
		_ => return Err(Error::NoChildren),
	}
	Ok(())
}

// Extract single element of sexpr at index i
pub fn pop(v: &mut Lval, i: usize) -> BlisprResult {
	match *v {
		Lval::Sexpr(ref mut children)
		| Lval::Qexpr(ref mut children)
		| Lval::Blispr(ref mut children) => {
			let ret = (&children[i]).clone();
			children.remove(i);
			Ok(ret)
		},
		_ => Err(Error::NoChildren),
	}
}

// Add each cell in y to x
pub fn join(x: &mut Lval, mut y: Box<Lval>) -> Result<()> {
	while y.len()? > 0 {
		add(x, &*pop(&mut y, 0)?)?;
	}
	Ok(())
}
