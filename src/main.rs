#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate rustyline;

use pest::{Pairs, Parser};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fmt;

// First get the calculator working

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("simple.pest");

#[derive(Parser)]
#[grammar = "simple.pest"]
struct BlisprParser;

enum LVAL {
    LVAL_NUM(i32),
    LVAL_SYM(String), // &'a str!
}

// PRINT

impl fmt::Display for LVAL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::LVAL::*;
        match self {
            LVAL_NUM(n) => write!(f, "{}", n),
            LVAL_SYM(s) => write!(f, "{}", s),
            _ => write!(f, "Unknown lval type!?"),
        }
    }
}

// READ

fn lval_read<'a>(expr: Pairs<Rule>) -> Box<LVAL> {
    Box::new(LVAL::LVAL_NUM(1))
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
                // match type - for now im leaving out the recursive ones
                // ast is a Pair -
                println!("{:#?}", ast);
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
