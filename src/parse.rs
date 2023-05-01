use crate::{
	error::{BlisprResult, Result},
	eval::lval_eval,
	lenv::Lenv,
	lval::{add, blispr, num, qexpr, sexpr, sym, Lval},
};
use log::debug;
use pest::{iterators::Pair, Parser};

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

// Read a rule with children into the given containing Lval
fn read_to_lval(v: &mut Lval, parsed: Pair<Rule>) -> Result<()> {
	for child in parsed.into_inner() {
		if is_bracket_or_eoi(&child) {
			continue;
		}
		add(v, &*lval_read(child)?)?;
	}
	Ok(())
}

fn lval_read(parsed: Pair<Rule>) -> BlisprResult {
	match parsed.as_rule() {
		Rule::blispr => {
			let mut ret = blispr();
			read_to_lval(&mut ret, parsed)?;
			Ok(ret)
		},
		Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
		Rule::sexpr => {
			let mut ret = sexpr();
			read_to_lval(&mut ret, parsed)?;
			Ok(ret)
		},
		Rule::qexpr => {
			let mut ret = qexpr();
			read_to_lval(&mut ret, parsed)?;
			Ok(ret)
		},
		Rule::num => Ok(num(parsed.as_str().parse::<i64>()?)),
		Rule::symbol => Ok(sym(parsed.as_str())),
		_ => unreachable!(), // COMMENT/WHITESPACE etc
	}
}

pub fn eval_str(e: &mut Lenv, s: &str) -> BlisprResult {
	let parsed = BlisprParser::parse(Rule::blispr, s)?.next().unwrap();
	debug!("{}", parsed);
	let mut lval_ptr = lval_read(parsed)?;
	debug!("Parsed: {:?}", *lval_ptr);
	lval_eval(e, &mut lval_ptr)
}
