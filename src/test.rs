// These are integration tests - I'm not clear on how best to unit test this yet
use crate::{lenv::Lenv, lval::Lval, parse::eval_str};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
fn test_blispr(test_str: &str, expected: &Lval) {
	assert_eq!(
		&*eval_str(&mut Lenv::new(None, None), test_str).unwrap(),
		expected
	);
}

#[test]
fn test_add_two_numbers() {
	test_blispr("(+ 1 2)", &Lval::Num(3));
}

#[test]
fn test_add_three_numbers() {
	test_blispr("(+ 1 2 3)", &Lval::Num(6));
}

#[test]
fn test_sub_two_numbers() {
	test_blispr("(- 1 2)", &Lval::Num(-1));
}

#[test]
fn test_sub_three_numbers() {
	test_blispr("(- 1 2 3)", &Lval::Num(-4));
}

#[test]
fn test_mul_two_numbers() {
	test_blispr("(* 2 3)", &Lval::Num(6));
}

#[test]
fn test_mul_three_numbers() {
	test_blispr("(* 2 3 4)", &Lval::Num(24));
}

#[test]
fn test_div_two_numbers() {
	test_blispr("(/ 4 2)", &Lval::Num(2));
}

#[test]
fn test_div_three_numbers() {
	test_blispr("(/ 16 4 2)", &Lval::Num(2));
}

#[test]
fn test_pow_two_numbers() {
	test_blispr("(^ 2 4)", &Lval::Num(16));
}

#[test]
fn test_pow_three_numbers() {
	test_blispr("(^ 2 4 4)", &Lval::Num(65536));
}

#[test]
fn test_rem_two_numbers() {
	test_blispr("(% 23 5)", &Lval::Num(3));
}

#[test]
fn test_rem_three_numbers() {
	test_blispr("(% 23 5 2)", &Lval::Num(1));
}

#[test]
fn test_max_two_numbers() {
	test_blispr("(max 1 2)", &Lval::Num(2));
}

#[test]
fn test_max_three_numbers() {
	test_blispr("(max 1 3 2)", &Lval::Num(3));
}

#[test]
fn test_min_two_numbers() {
	test_blispr("(min 1 2)", &Lval::Num(1));
}

#[test]
fn test_min_three_numbers() {
	test_blispr("(min 1 3 2)", &Lval::Num(1));
}

#[test]
fn test_head() {
	test_blispr("(head {1 2 3})", &Lval::Num(1));
}

#[test]
fn test_tail() {
	test_blispr(
		"(tail {1 2 3})",
		&Lval::Qexpr(vec![Box::new(Lval::Num(2)), Box::new(Lval::Num(3))]),
	);
}

#[test]
fn test_cons() {
	test_blispr(
		"(cons 3 {4 5})",
		&Lval::Qexpr(vec![
			Box::new(Lval::Num(3)),
			Box::new(Lval::Num(4)),
			Box::new(Lval::Num(5)),
		]),
	);
}

#[test]
fn test_len() {
	test_blispr("(len {1 2 3})", &Lval::Num(3));
}

#[test]
fn test_list() {
	test_blispr(
		"(list 1 2 3)",
		&Lval::Qexpr(vec![
			Box::new(Lval::Num(1)),
			Box::new(Lval::Num(2)),
			Box::new(Lval::Num(3)),
		]),
	);
}

#[test]
fn test_join() {
	test_blispr(
		"(join {1 2} {2 3})",
		&Lval::Qexpr(vec![
			Box::new(Lval::Num(1)),
			Box::new(Lval::Num(2)),
			Box::new(Lval::Num(2)),
			Box::new(Lval::Num(3)),
		]),
	);
}

#[test]
fn test_init() {
	test_blispr(
		"(init {1 2 3})",
		&Lval::Qexpr(vec![Box::new(Lval::Num(1)), Box::new(Lval::Num(2))]),
	);
}

#[test]
fn test_eval() {
	test_blispr("(eval {+ 1 2})", &Lval::Num(3));
}

#[test]
fn test_unary_negation() {
	test_blispr("(- 3)", &Lval::Num(-3));
}

#[test]
fn test_two_forms() {
	test_blispr("(+ 1 2)(+ 2 3)", &Lval::Num(5));
}

#[test]
fn test_def() {
	test_blispr("(def {x} 12)x", &Lval::Num(12));
}

#[test]
fn test_def_multi() {
	test_blispr("(def {a b} 1 2)(+ a b)", &Lval::Num(3));
}

#[test]
fn test_lambda() {
	test_blispr("((\\ {x y} {+ x y}) 2 3)", &Lval::Num(5));
}

#[test]
fn test_def_lambda() {
	test_blispr("(def {func} (\\ {x y} {+ x y}))(func 5 6)", &Lval::Num(11));
}

#[test]
fn test_partial_application() {
	test_blispr(
		"(def {func} (\\ {x y} {+ x y}))(def {func-2} (func 2))(func-2 7)",
		&Lval::Num(9),
	);
}

#[test]
fn test_input_file() {
	let file_str = include_str!("../test.blispr");
	test_blispr(file_str, &Lval::Num(311));
}
