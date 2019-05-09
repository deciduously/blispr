// These are integration tests - I'm not clear on how best to unit test this yet
use crate::{lenv::Lenv, lval::Lval, parse::eval_str};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_hello_world() {
    let mut env = &mut Lenv::new(None, None);

    assert_eq!(*eval_str(&mut env, "(+ 1 2)").unwrap(), Lval::Num(3))
}
