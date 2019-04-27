use std::fmt;

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren<'a> = Vec<Box<Lval<'a>>>;

// The main type - all possible Blispr values
#[derive(Debug, Clone)]
pub enum Lval<'a> {
    Err(&'a str),
    Num(i64),
    Sym(&'a str),
    Sexpr(LvalChildren<'a>),
    Qexpr(LvalChildren<'a>),
}

impl<'a> Lval<'a> {
    pub fn as_num(&self) -> Result<i64, Box<Lval<'a>>> {
        match *self {
            Lval::Num(n_num) => Ok(n_num),
            _ => Err(lval_err("Not a number!")),
        }
    }
}

impl<'a> fmt::Display for Lval<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lval::Err(e) => write!(f, "Error: {}", e),
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

pub fn lval_err<'a>(e_str: &'a str) -> Box<Lval<'a>> {
    Box::new(Lval::Err(e_str))
}

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
pub fn lval_add<'a>(v: &mut Lval<'a>, x: Box<Lval<'a>>) {
    match *v {
        Lval::Err(_) | Lval::Num(_) | Lval::Sym(_) => {
            panic!("Tried to add a child to a non-containing lval!")
        }
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            children.push(x);
        }
    }
}

// Extract single element of sexpr at index i
pub fn lval_pop<'a>(v: &mut Lval<'a>, i: usize) -> Box<Lval<'a>> {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            ret
        }
        _ => lval_err("Cannot pop from a non-containing lval!"),
    }
}
