use crate::lval::{
    lval_add, lval_err, lval_join, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval,
};
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
        debug!("Unary negation on {}", x);
        let x_num = x.as_num()?;
        return Ok(lval_num(-x_num));
    }

    // consume the children until empty
    // and operate on x
    while child_count > 1 {
        let y = lval_pop(&mut v, 0);
        child_count -= 1;
        match func {
            "+" | "add" => {
                debug!("Add {} and {}", x, y);
                apply_binop!(add, x, y)
            }
            "-" | "sub" => {
                debug!("Subtract {} and {}", x, y);
                apply_binop!(sub, x, y)
            }
            "*" | "mul" => {
                debug!("Multiply {} and {}", x, y);
                apply_binop!(mul, x, y)
            }
            "/" | "div" => {
                if y.as_num()? == 0 {
                    debug!("Failed divide {} by {}", x, y);
                    return Err(lval_err("Divide by zero!"));
                } else {
                    debug!("Divide {} by {}", x, y);
                    apply_binop!(div, x, y)
                }
            }
            "%" | "rem" => {
                debug!("{} % {}", x, y);
                apply_binop!(rem, x, y)
            }
            "^" | "pow" => {
                debug!("Raise {} to the {} power", x, y);
                let y_num = y.as_num()?;
                let x_num = x.as_num()?;
                let mut coll = 1;
                for _ in 0..y_num {
                    coll *= x_num;
                }
                x = lval_num(coll);
            }
            "min" => {
                debug!("Min {} and {}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num < y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            "max" => {
                debug!("Max {} and {}", x, y);
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
        Lval::Qexpr(ref children) => {
            let mut new_sexpr = lval_sexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_sexpr, cloned);
            }
            debug!("builtin_eval: {:?}", new_sexpr);
            lval_eval(new_sexpr)
        }
        _ => {
            debug!("Failed builtin_eval on {:?}", v);
            Err(lval_err("Tried to eval a non-qexpr"))
        }
    }
}

// Join the children into one qexpr
fn builtin_join<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    let mut ret = lval_qexpr();
    for _ in 0..v.len()? {
        let next = lval_pop(&mut v, 0);
        match *next {
            Lval::Qexpr(_) => {
                lval_join(&mut ret, next)?;
            }
            _ => return Err(lval_err("non-Qexpr arg passed to join")),
        }
    }
    Ok(ret)
}

// make sexpr into a qexpr
fn builtin_list<'a>(v: Box<Lval<'a>>) -> Box<Lval<'a>> {
    match *v {
        Lval::Sexpr(ref children) => {
            debug!("Building list from {:?}", children);
            let mut new_qexpr = lval_qexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_qexpr, cloned);
            }
            new_qexpr
        }
        _ => v,
    }
}

fn builtin_len<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    if v.len()? != 1 {
        return Err(lval_err("len called with more than one argument"));
    }
    let qexpr = lval_pop(&mut v, 0);
    match *qexpr {
        Lval::Qexpr(_) => Ok(lval_num(qexpr.len()? as i64)),
        _ => Err(lval_err("len called on something that isn't a list")),
    }
}

fn builtin<'a>(mut v: Box<Lval<'a>>, func: &str) -> Result<Box<Lval<'a>>, Box<Lval<'a>>> {
    match func {
        "+" | "-" | "*" | "/" | "%" | "^" | "add" | "sub" | "mul" | "div" | "rem" | "pow"
        | "max" | "min" => builtin_op(v, func),
        "eval" => {
            // Unwrap the containing Sexpr
            let qexpr = lval_pop(&mut v, 0);
            builtin_eval(qexpr)
        }
        "join" => builtin_join(v),
        "len" => builtin_len(v),
        "list" => Ok(builtin_list(v)),
        _ => Err(lval_err("Unknown function!")),
    }
}

pub fn lval_eval(mut v: Box<Lval>) -> Result<Box<Lval>, Box<Lval>> {
    let child_count;
    match *v {
        Lval::Sexpr(ref mut cells) => {
            debug!("lval_eval: Sexpr({:?})", cells);
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
        _ => {
            debug!("lval_eval: Non-sexpr: {:?}", v);
            return Ok(v);
        }
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
        debug!("Calling function {} on {:?}", lfn, v);
        match *lfn {
            Lval::Sym(s) => builtin(v, &s),
            _ => {
                println!("{}", *lfn);
                Err(lval_err("S-expression does not start with symbol"))
            }
        }
    }
}
