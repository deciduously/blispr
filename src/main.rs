#[macro_use]
extern crate pest_derive;

mod eval;
mod lval;

use eval::lval_eval;
use lval::{lval_add, lval_num, lval_qexpr, lval_sexpr, lval_sym, Lval};
use pest::{iterators::Pair, Parser};
use rustyline::{error::ReadlineError, Editor};

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blispr.pest");

#[derive(Parser)]
#[grammar = "blispr.pest"]
pub struct BlisprParser;

fn is_bracket_or_eoi(parsed: &Pair<Rule>) -> bool {
    if parsed.as_rule() == Rule::EOI {
        return true;
    }
    let c = parsed.as_str();
    c == "(" || c == ")" || c == "{" || c == "}"
}

pub fn lval_read(parsed: Pair<Rule>) -> Box<Lval> {
    // TODO skip brackets and such
    match parsed.as_rule() {
        Rule::blispr | Rule::sexpr => {
            let mut ret = lval_sexpr();
            for child in parsed.into_inner() {
                // here is where you skip stuff
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                ret = lval_add(ret, lval_read(child));
            }
            ret
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::qexpr => {
            let mut ret = lval_qexpr(Vec::new());
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                ret = lval_add(ret, lval_read(child));
            }
            ret
        }
        Rule::num => lval_num(parsed.as_str().parse::<i64>().unwrap()),
        Rule::symbol => lval_sym(parsed.as_str()),
        _ => unreachable!(),
    }
}

fn repl(print_parsed: bool) {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D to exit");
    if print_parsed {
        println!("Debug mode enabled");
    }

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
                let lval_ptr = lval_read(parsed);
                if print_parsed {
                    println!("{}", *lval_ptr);
                }
                let res = match lval_eval(lval_ptr) {
                    Ok(r) => r,
                    Err(e) => e,
                };
                println!("{}", res);
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

fn main() {
    // set "-p" to also print out the parsed blipsr, pre-eval
    // first check if we have a flag at all
    let print_parsed = {
        let args = &::std::env::args().collect::<Vec<String>>();

        if args.len() == 1 {
            false
        } else {
            args[1] == "-p"
        }
    };

    repl(print_parsed);
}
