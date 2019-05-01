use crate::{
    error::{BlisprResult, Result},
    eval::lval_eval,
    lenv::ENV,
    lval::{lval_add, lval_num, lval_qexpr, lval_sexpr, lval_sym},
};
use pest::{iterators::Pair, Parser};
use std::sync::Arc;

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

fn lval_read(parsed: Pair<Rule>) -> BlisprResult {
    match parsed.as_rule() {
        Rule::blispr | Rule::sexpr => {
            let mut ret = lval_sexpr();
            for child in parsed.into_inner() {
                // here is where you skip stuff
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::qexpr => {
            let mut ret = lval_qexpr();
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::num => Ok(lval_num(parsed.as_str().parse::<i64>()?)),
        Rule::symbol => Ok(lval_sym(parsed.as_str())),
        _ => unreachable!(), // COMMENT/WHITESPACE etc
    }
}

pub fn eval_str(s: &str) -> Result<()> {
    let parsed = BlisprParser::parse(Rule::blispr, s)?.next().unwrap();
    debug!("{}", parsed);
    let lval_ptr = lval_read(parsed)?;
    debug!("Parsed: {:?}", *lval_ptr);
    let env_arc = Arc::clone(&ENV);
    let r = env_arc.read()?;
    println!("{}", lval_eval(Box::new(*r), lval_ptr)?);
    Ok(())
}
