extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate rustyline;

use pest::{iterators::Pair, Parser};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fmt;

// First get the calculator working

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blisp.pest");

#[derive(Parser)]
#[grammar = "blisp.pest"]
struct BlisprParser;

#[derive(Clone)]
enum Lval {
    Err(String),
    Num(i64),
    Sym(String), // &'a str?
    // TODO Shoudl this be a VecDeque or a LinkedList?
    Sexpr(Vec<Box<Lval>>),
    Qexpr(Vec<Box<Lval>>),
    Blispr(Vec<Box<Lval>>),
}

// PRINT

impl fmt::Display for Lval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lval::Err(e) => write!(f, "Error: {}", e),
            Lval::Num(n) => write!(f, "{}", n),
            Lval::Sym(s) => write!(f, "{}", s),
            Lval::Sexpr(cell) => write!(f, "({})", lval_expr_print(cell)),
            Lval::Qexpr(cell) => write!(f, "'({})", lval_expr_print(cell)),
            Lval::Blispr(cell) => write!(f, "{}", lval_expr_print(cell)),
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

fn lval_read(parsed: Pair<Rule>) -> Box<Lval> {
    match parsed.as_rule() {
        Rule::blispr => {
            //println!("Making toplevel!");
            let mut ret = Vec::new();
            for child in parsed.into_inner() {
                ret.push(lval_read(child));
            }
            Box::new(Lval::Blispr(ret))
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::sexpr => {
            //println!("making sexpr!");
            let mut ret = Vec::new();
            for child in parsed.into_inner() {
                ret.push(lval_read(child));
            }
            Box::new(Lval::Sexpr(ret))
        }
        Rule::qexpr => {
            //println!("making qexpr!");
            let mut ret = Vec::new();
            for child in parsed.into_inner() {
                ret.push(lval_read(child));
            }
            Box::new(Lval::Qexpr(ret))
        }
        Rule::num => {
            let num = parsed.as_str().parse::<i64>().unwrap();
            //println!("int | digit: {}", num);
            Box::new(Lval::Num(num))
        }
        Rule::symbol => {
            let sym = parsed.as_str();
            //println!("symbol: {}", sym);
            Box::new(Lval::Sym(sym.into()))
        }
        Rule::comment | Rule::whitespace => unimplemented!(),
        Rule::int | Rule::digit => unimplemented!(), // should never hit - num will cover it?
    }
}

// EVAL

fn builtin(v: &Box<Lval>, func: &str) -> Box<Lval> {
    unimplemented!()
}

fn lval_eval_sexpr(v: &mut Box<Lval>) -> &mut Box<Lval> {
    // Don't be afraid to clone and drop the old one.Err
    // Look at the C lval_add and stuff like that
    match **v {
        Lval::Sexpr(ref mut cells) => {
            // First, evaluate all the cells inside
            let length = cells.len();
            for i in 0..length {
                let child = cells[i].clone();
                cells[i] = *lval_eval(&mut child);
            };

            // Error checking
            // if any is an error, return an Lval::Err
            for i in 0..length {
                match *cells[i] {
                    Lval::Err(s) => return &mut Box::new(Lval::Err(s)),
                    _ => continue,
                }
            }
        
            if length == 0 {
                // Empty expression
                &mut Box::new(*v.clone())
            } else if length == 1 {
                // Single expression
                &mut cells[0].clone()
            } else {
                // Function call
                // Ensure the first element is a Symbol
                let lfn = *cells[0].clone();
                match lfn {
                    Lval::Sym(s) => &mut builtin(&v, &s),
                    _ => return &mut Box::new(Lval::Err("S-expression does not start with symbol!".into())),
                }
            }
        }
        _ => return &mut Box::new(Lval::Err("lval_eval_sexpr called on something that's not a sexpr!  Why you gotta do me like that".into())),
    }
}

fn lval_eval(v: &mut Box<Lval>) -> &mut Box<Lval> {
    match **v {
        Lval::Sexpr(_) => lval_eval_sexpr(&mut v),
        _ => v,
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
                let ast = BlisprParser::parse(Rule::blispr, &line)
                    .expect("Syntax error!")
                    .next()
                    .unwrap();
                println!("{}", lval_eval(&mut lval_read(ast)));
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
