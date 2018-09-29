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
        // I think I've now got an iterator over the exprs in the blisp
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

//fn print_ast()

fn main() {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D to exit");

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let ast = BlisprParser::parse(Rule::blispr, &line)
                    .expect("Gibberish!  Try some real blispr next time")
                    .next()
                    .unwrap();
                //println!("{:#?}", ast.into_inner());
                println!("{}", lval_read(ast));
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
    rl.save_history("history.txt").unwrap();
}
