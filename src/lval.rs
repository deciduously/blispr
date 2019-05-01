use crate::{
    error::{BlisprError, BlisprResult, Result},
    lenv::Lenv,
};
use std::fmt;

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren = Vec<Box<Lval>>;
pub type LBuiltin = fn(Box<Lval>) -> BlisprResult;

// There are two types of function - builtin and lambda
#[derive(Debug, Clone, PartialEq)]
pub enum LvalFun {
    Builtin(LBuiltin),                       // (function pointer)
    Lambda(Box<Lenv>, Box<Lval>, Box<Lval>), // (environment, formals, body), both should be Qexpr
}

// The main type - all possible Blispr values
#[derive(Debug, Clone, PartialEq)]
pub enum Lval {
    Fun(LvalFun),
    Num(i64),
    Sym(String),
    Sexpr(LvalChildren),
    Qexpr(LvalChildren),
}

impl Lval {
    pub fn as_num(&self) -> Result<i64> {
        match *self {
            Lval::Num(n_num) => Ok(n_num),
            _ => Err(BlisprError::NotANumber),
        }
    }
    pub fn as_string(&self) -> Result<String> {
        match self {
            Lval::Sym(s) => Ok(s.to_string()),
            _ => Err(BlisprError::WrongType(
                "symbol".to_string(),
                format!("{:?}", self),
            )),
        }
    }
    pub fn len(&self) -> Result<usize> {
        match *self {
            Lval::Sexpr(ref children) | Lval::Qexpr(ref children) => Ok(children.len()),
            _ => Err(BlisprError::NoChildren),
        }
    }
}

impl fmt::Display for Lval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lval::Fun(lf) => match lf {
                LvalFun::Builtin(_) => write!(f, "<builtin>"),
                LvalFun::Lambda(_, formals, body) => write!(f, "(\\ {} {})", formals, body),
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

pub fn lval_builtin(f: LBuiltin) -> Box<Lval> {
    Box::new(Lval::Fun(LvalFun::Builtin(f)))
}

pub fn lval_lambda(formals: Box<Lval>, body: Box<Lval>) -> Box<Lval> {
    Box::new(Lval::Fun(LvalFun::Lambda(
        Box::new(Lenv::new(None)),
        formals,
        body,
    )))
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
pub fn lval_add(v: &mut Lval, x: Box<Lval>) -> Result<()> {
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
pub fn lval_join(x: &mut Lval, mut y: Box<Lval>) -> Result<()> {
    while y.len()? > 0 {
        lval_add(x, lval_pop(&mut y, 0)?)?;
    }
    Ok(())
}
