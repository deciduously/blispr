use crate::{
    error::{BlisprError, BlisprResult},
    lenv::LenvT,
};
use std::fmt;

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren = Vec<Box<Lval>>;
pub type LBuiltin = fn(LenvT, Box<Lval>) -> BlisprResult;

// The main type - all possible Blispr values
#[derive(Debug, Clone)]
pub enum Lval {
    Fun(LBuiltin),
    Num(i64),
    Sym(String),
    Sexpr(LvalChildren),
    Qexpr(LvalChildren),
}

impl Lval {
    pub fn as_num(&self) -> Result<i64, BlisprError> {
        match *self {
            Lval::Num(n_num) => Ok(n_num),
            _ => Err(BlisprError::NotANumber),
        }
    }
    pub fn len(&self) -> Result<usize, BlisprError> {
        match *self {
            Lval::Sexpr(ref children) | Lval::Qexpr(ref children) => Ok(children.len()),
            _ => Err(BlisprError::NoChildren),
        }
    }
}

impl fmt::Display for Lval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lval::Fun(_) => write!(f, "<function>"),
            Lval::Num(n) => write!(f, "{}", n),
            Lval::Sym(s) => write!(f, "{}", s),
            Lval::Sexpr(cell) => write!(f, "({})", lval_expr_print(cell)),
            Lval::Qexpr(cell) => write!(f, "{{{}}}", lval_expr_print(cell)),
        }
    }
}

impl PartialEq for Lval {
    fn eq(&self, other: &Lval) -> bool {
        match self {
            Lval::Fun(_) => false, // for now?  how to compare functions
            Lval::Num(contents) => match other {
                Lval::Num(other_contents) => contents == other_contents,
                _ => false,
            },
            Lval::Sym(contents) => match other {
                Lval::Sym(other_contents) => contents == other_contents,
                _ => false,
            },
            Lval::Sexpr(contents) => match other {
                Lval::Sexpr(other_contents) => contents == other_contents,
                _ => false,
            },
            Lval::Qexpr(contents) => match other {
                Lval::Qexpr(other_contents) => contents == other_contents,
                _ => false,
            },
        }
    }
}

fn lval_expr_print(cell: &[Box<Lval>]) -> String {
    let mut ret = String::new();
    for i in 0..cell.len() {
        ret.push_str(&format!("{}", cell[i]));
        if i < cell.len() - 1 {
            ret.push_str(" ");
        }
    }
    ret
}

// Constructors
// Each allocates a brand new boxed Lval
// The recursive types start empty

// You can omit the lifetime annotations when the constructor is passed a reference
// I included them for consistency

pub fn lval_fun(f: LBuiltin) -> Box<Lval> {
    Box::new(Lval::Fun(f))
}

pub fn lval_num(n: i64) -> Box<Lval> {
    Box::new(Lval::Num(n))
}

pub fn lval_sym(s: &str) -> Box<Lval> {
    Box::new(Lval::Sym(s.into()))
}

pub fn lval_sexpr() -> Box<Lval> {
    Box::new(Lval::Sexpr(Vec::new()))
}

pub fn lval_qexpr() -> Box<Lval> {
    Box::new(Lval::Qexpr(Vec::new()))
}

// Manipulating children

// Add lval x to lval::sexpr or lval::qexpr v
pub fn lval_add(v: &mut Lval, x: Box<Lval>) -> Result<(), BlisprError> {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            children.push(x);
        }
        _ => return Err(BlisprError::NoChildren),
    }
    Ok(())
}

// Extract single element of sexpr at index i
pub fn lval_pop(v: &mut Lval, i: usize) -> BlisprResult {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            Ok(ret)
        }
        _ => Err(BlisprError::NoChildren),
    }
}

// Add each cell in y to x
pub fn lval_join(x: &mut Lval, mut y: Box<Lval>) -> Result<(), BlisprError> {
    while y.len()? > 0 {
        lval_add(x, lval_pop(&mut y, 0)?)?;
    }
    Ok(())
}
