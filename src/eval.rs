use crate::lval::{lval_err, lval_num, lval_pop, lval_qexpr, Lval};
use std::ops::{Add, Div, Mul, Rem, Sub};

macro_rules! apply_binop {
    ( $op:ident, $x:ident, $y:ident ) => {
        match (*$x, *$y) {
            (Lval::Num(x_num), Lval::Num(y_num)) => {
                $x = lval_num(x_num.$op(y_num));
                continue;
            }
            _ => return Err(lval_err("Not a number")), // TODO error type
        }
    };
}

fn builtin_op<'a>(mut v: Box<Lval<'a>>, func: &str) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    let mut child_count;
    match *v {
        Lval::Sexpr(ref children) => {
            child_count = children.len();
        }
        _ => return Ok(v),
    }

    let mut x = lval_pop(&mut v, 0);

    // If no args given and we're doing subtraction, perform unary negation
    if (func == "-" || func == "sub") && child_count == 1 {
        let x_num = x.as_num()?;
        return Ok(lval_num(-x_num));
    }

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
                if y.as_num()? == 0 {
                    return Err(lval_err("Divide by zero!"));
                } else {
                    apply_binop!(div, x, y)
                }
            }
            "%" | "rem" => apply_binop!(rem, x, y),
            "^" | "pow" => {
                let y_num = y.as_num()?;
                let x_num = x.as_num()?;
                let mut coll = 1;
                for _ in 0..y_num {
                    coll *= x_num;
                }
                x = lval_num(coll);
            }
            "min" => {
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num < y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            "max" => {
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num > y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            _ => unreachable!(), // builtin() took care of it
        }
    }
    Ok(x)
}

// Evaluate qexpr as a sexpr
fn builtin_eval<'a>(v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    match *v {
        Lval::Qexpr(ref children) => lval_eval(Box::new(Lval::Sexpr(children.to_vec()))),
        _ => Ok(v),
    }
}

// Join the children into one qexpr
//fn builtin_join<'a>(v: Box<Lval<'a>>) -> Box<Lval<'a>> {
//    let mut child_count;
//    let x = lval_pop(&mut v, 0);
//
//    unimplemented!()
//match *x {
//Lval::Qexpr
//
//}
//}

// make sexpr into a qexpr
fn builtin_list<'a>(v: Box<Lval<'a>>) -> Box<Lval<'a>> {
    match *v {
        Lval::Sexpr(ref children) => lval_qexpr(children.to_vec()),
        _ => v,
    }
}

fn builtin<'a>(v: Box<Lval<'a>>, func: &str) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    match func {
        "+" | "-" | "*" | "/" | "%" | "^" | "add" | "sub" | "mul" | "div" | "rem" | "pow"
        | "max" | "min" => builtin_op(v, func),
        "eval" => builtin_eval(v),
        //"join" => builtin_join(v),
        "list" => Ok(builtin_list(v)),
        _ => Err(lval_err("Unknown function!")),
    }
}

pub fn lval_eval(mut v: Box<Lval>) -> Result<Box<Lval>, Box<Lval>> {
    let child_count;
    match *v {
        Lval::Sexpr(ref mut cells) => {
            // First, evaluate all the cells inside
            child_count = cells.len();
            for item in cells.iter_mut().take(child_count) {
                *item = lval_eval(item.clone())?
            }

            // Error checking
            // if any is an error, return an Lval::Err
            for item in cells.iter().take(child_count) {
                let res = *item.clone();
                match res {
                    Lval::Err(s) => return Err(lval_err(s)),
                    _ => continue,
                }
            }
        }
        // if it's not a sexpr, we're done
        _ => return Ok(v),
    }

    if child_count == 0 {
        // It was a sexpr, but it was empty
        Ok(v)
    } else if child_count == 1 {
        // Single expression
        Ok(lval_pop(&mut v, 0))
    } else {
        // Function call
        // Ensure the first element is a Symbol
        let lfn = lval_pop(&mut v, 0);
        match *lfn {
            Lval::Sym(s) => builtin(v, &s),
            _ => {
                println!("{}", *lfn);
                Err(lval_err("S-expression does not start with symbol"))
            }
        }
    }
}
