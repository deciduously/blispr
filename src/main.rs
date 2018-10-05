extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate rustyline;

use pest::{iterators::Pair, Parser};
use rustyline::{error::ReadlineError, Editor};
use std::{
    fmt,
    ops::{Add, Div, Mul, Rem, Sub},
};

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blispr.pest");

#[derive(Parser)]
#[grammar = "blispr.pest"]
struct BlisprParser;

macro_rules! apply_binop {
    ( $op:ident, $x:ident, $y:ident ) => {
        match *$x {
            Lval::Num(x_num) => match *$y {
                Lval::Num(y_num) => {
                    $x = lval_num(x_num.$op(y_num));
                    continue;
                }
                _ => return lval_err("Not a number"), // TODO error type
            },
            _ => return lval_err("Not a number"),
        }
    };
}

macro_rules! lval_num_inner {
    ( $lval_n:ident ) => {
        match *$lval_n {
            Lval::Num(n_num) => n_num,
            _ => return lval_err("Not a number"),
        }
    };
}

// The recursive types hold their children in one of these bad boys
// TODO Should this be a VecDeque or a LinkedList instead?
type LvalChildren<'a> = Vec<Box<Lval<'a>>>;

// The main type - all possible Blispr values
#[derive(Debug, Clone)]
enum Lval<'a> {
    Err(&'a str),
    Num(i64),
    Sym(&'a str),
    Sexpr(LvalChildren<'a>),
    Qexpr(LvalChildren<'a>),
}

// Constructors
// Each allocates a brand new boxed Lval
// The recursive types start empty

// You can omit the lifetime annotations when the constructor is passed a reference
// I included them for consistency

fn lval_err<'a>(e_str: &'a str) -> Box<Lval<'a>> {
    Box::new(Lval::Err(e_str))
}

fn lval_num<'a>(n: i64) -> Box<Lval<'a>> {
    Box::new(Lval::Num(n))
}

fn lval_sym<'a>(s: &'a str) -> Box<Lval<'a>> {
    Box::new(Lval::Sym(s))
}

fn lval_sexpr<'a>() -> Box<Lval<'a>> {
    Box::new(Lval::Sexpr(Vec::new()))
}

fn lval_qexpr<'a>() -> Box<Lval<'a>> {
    Box::new(Lval::Qexpr(Vec::new()))
}

// Manipluating children

// Add lval x to lval::sexpr v
// Takes ownership of both which drops them, and returns a brand new Box<Lval> instead of mutating v
fn lval_add<'a>(v: &Lval<'a>, x: Box<Lval<'a>>) -> Box<Lval<'a>> {
    match *v {
        Lval::Err(_) | Lval::Num(_) | Lval::Sym(_) => {
            panic!("Tried to add a child to a non-containing lval!")
        }
        Lval::Sexpr(ref children) | Lval::Qexpr(ref children) => {
            let mut new_children = children.clone();
            new_children.push(x);
            Box::new(Lval::Sexpr(new_children))
        }
    }
}

// Extract single element of sexpr at index i
fn lval_pop<'a>(v: &mut Lval<'a>, i: usize) -> Box<Lval<'a>> {
    match *v {
        Lval::Sexpr(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            ret
        }
        _ => lval_err("Cannot pop from a non-sexpr lval!"),
    }
}

// PRINT

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

// READ

fn is_bracket_or_eoi(parsed: &Pair<Rule>) -> bool {
    if parsed.as_rule() == Rule::EOI {
        return true;
    }
    let c = parsed.as_str();
    c == "(" || c == ")" || c == "{" || c == "}"
}

fn lval_read(parsed: Pair<Rule>) -> Box<Lval> {
    // TODO skip brackets and such
    match parsed.as_rule() {
        Rule::blispr | Rule::sexpr => {
            let mut ret = lval_sexpr();
            for child in parsed.into_inner() {
                // here is where you skip stuff
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                ret = lval_add(&ret, lval_read(child));
            }
            ret
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::qexpr => {
            let mut ret = lval_qexpr();
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                ret = lval_add(&ret, lval_read(child));
            }
            ret
        }
        Rule::num => lval_num(parsed.as_str().parse::<i64>().unwrap()),
        Rule::symbol => lval_sym(parsed.as_str()),
        _ => unreachable!(),
    }
}

// EVAL

fn builtin_op<'a>(mut v: Box<Lval<'a>>, func: &str) -> Box<Lval<'a>> {
    // TODO check all args are numbers first?

    let mut child_count;
    match *v {
        Lval::Sexpr(ref children) => {
            child_count = children.len();
        }
        _ => return v,
    }
    let mut x = lval_pop(&mut v, 0);
    // If no args given and we're doing subtraction, perform unary negation
    // TODO

    // consume the children until empty
    // and operate on x
    while child_count > 1 {
        let y = lval_pop(&mut v, 0);
        child_count -= 1;
        match func {
            "+" | "add" => apply_binop!(add, x, y),
            "-" | "sub" => apply_binop!(sub, x, y),
            "*" | "mul" => apply_binop!(mul, x, y),
            "/" | "div" => {
                if lval_num_inner!(y) == 0 {
                    return lval_err("Divide by zero!");
                } else {
                    apply_binop!(div, x, y)
                }
            }
            "%" | "rem" => apply_binop!(rem, x, y),
            "min" => {
                let x_num = lval_num_inner!(x);
                let y_num = lval_num_inner!(y);
                if x_num > y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            "max" => {
                let x_num = lval_num_inner!(x);
                let y_num = lval_num_inner!(y);
                if x_num < y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            _ => unreachable!(), // builtin() took care of it
        }
    }
    x
}

fn builtin<'a>(v: Box<Lval<'a>>, func: &str) -> Box<Lval<'a>> {
    match func {
        "+" | "-" | "*" | "/" | "%" | "add" | "sub" | "mul" | "div" | "rem" | "max" | "min" => {
            builtin_op(v, func)
        }
        _ => lval_err("Unknown function!"),
    }
}

fn lval_eval(mut v: Box<Lval>) -> Box<Lval> {
    let child_count;
    match *v {
        Lval::Sexpr(ref mut cells) => {
            // First, evaluate all the cells inside
            child_count = cells.len();
            for item in cells.iter_mut().take(child_count) {
                *item = lval_eval(item.clone())
            }

            // Error checking
            // if any is an error, return an Lval::Err
            for item in cells.iter().take(child_count) {
                let res = *item.clone();
                match res {
                    Lval::Err(s) => return lval_err(s),
                    _ => continue,
                }
            }
        }
        // if it's not a sexpr, we're done
        _ => return v,
    }

    if child_count == 0 {
        // It was a sexpr, but it was empty
        v
    } else if child_count == 1 {
        // Single expression
        lval_pop(&mut v, 0)
    } else {
        // Function call
        // Ensure the first element is a Symbol
        let lfn = lval_pop(&mut v, 0);
        match *lfn {
            Lval::Sym(s) => builtin(v, &s),
            _ => {
                println!("{}", *lfn);
                lval_err("S-expression does not start with symbol")
            }
        }
    }
}

fn main() {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D to exit");

    let mut rl = Editor::<()>::new();
    if rl.load_history("./.blisp-history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let parsed = BlisprParser::parse(Rule::blispr, &line)
                    .expect("Syntax error!")
                    .next()
                    .unwrap();
                println!("{}", lval_eval(lval_read(parsed)));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("./.blisp-history.txt").unwrap();
}
