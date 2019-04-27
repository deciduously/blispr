use crate::error::BlisprError;
use std::fmt;

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren<'a> = Vec<Box<Lval<'a>>>;

// The main type - all possible Blispr values
#[derive(Debug, Clone)]
pub enum Lval<'a> {
    Num(i64),
    Sym(&'a str),
    Sexpr(LvalChildren<'a>),
    Qexpr(LvalChildren<'a>),
}

impl<'a> Lval<'a> {
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

impl<'a> fmt::Display for Lval<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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

pub fn lval_num<'a>(n: i64) -> Box<Lval<'a>> {
    Box::new(Lval::Num(n))
}

pub fn lval_sym<'a>(s: &'a str) -> Box<Lval<'a>> {
    Box::new(Lval::Sym(s))
}

pub fn lval_sexpr<'a>() -> Box<Lval<'a>> {
    Box::new(Lval::Sexpr(Vec::new()))
}

pub fn lval_qexpr<'a>() -> Box<Lval<'a>> {
    Box::new(Lval::Qexpr(Vec::new()))
}

// Manipulating children

// Add lval x to lval::sexpr or lval::qexpr v
pub fn lval_add<'a>(v: &mut Lval<'a>, x: Box<Lval<'a>>) -> Result<(), BlisprError> {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            children.push(x);
        }
        _ => return Err(BlisprError::NoChildren),
    }
    Ok(())
}

// Extract single element of sexpr at index i
pub fn lval_pop<'a>(v: &mut Lval<'a>, i: usize) -> Result<Box<Lval<'a>>, BlisprError> {
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
pub fn lval_join<'a>(x: &mut Lval<'a>, mut y: Box<Lval<'a>>) -> Result<(), BlisprError> {
    while y.len()? > 0 {
        lval_add(x, lval_pop(&mut y, 0)?)?;
    }
    Ok(())
}
