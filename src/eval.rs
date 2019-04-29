use crate::{
    error::{BlisprError, BlisprResult},
    lenv::LenvT,
    lval::{lval_add, lval_join, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval},
};
use std::{
    ops::{Add, Div, Mul, Rem, Sub},
    sync::Arc,
};

macro_rules! apply_binop {
    ( $op:ident, $x:ident, $y:ident ) => {
        match (*$x, *$y) {
            (Lval::Num(x_num), Lval::Num(y_num)) => {
                $x = lval_num(x_num.$op(y_num));
                continue;
            }
            _ => return Err(BlisprError::NotANumber),
        }
    };
}

fn builtin_op(mut v: Box<Lval>, func: &str) -> BlisprResult {
    let mut child_count;
    match *v {
        Lval::Sexpr(ref children) => {
            child_count = children.len();
        }
        _ => return Ok(v),
    }

    let mut x = lval_pop(&mut v, 0)?;

    // If no args given and we're doing subtraction, perform unary negation
    if (func == "-" || func == "sub") && child_count == 1 {
        debug!("builtin_op: Unary negation on {}", x);
        let x_num = x.as_num()?;
        return Ok(lval_num(-x_num));
    }

    // consume the children until empty
    // and operate on x
    while child_count > 1 {
        let y = lval_pop(&mut v, 0)?;
        child_count -= 1;
        match func {
            "+" | "add" => {
                debug!("builtin_op: Add {} and {}", x, y);
                apply_binop!(add, x, y)
            }
            "-" | "sub" => {
                debug!("builtin_op: Subtract {} and {}", x, y);
                apply_binop!(sub, x, y)
            }
            "*" | "mul" => {
                debug!("builtin_op: Multiply {} and {}", x, y);
                apply_binop!(mul, x, y)
            }
            "/" | "div" => {
                if y.as_num()? == 0 {
                    debug!("builtin_op: Failed divide {} by {}", x, y);
                    return Err(BlisprError::DivideByZero);
                } else {
                    debug!("builtin_op: Divide {} by {}", x, y);
                    apply_binop!(div, x, y)
                }
            }
            "%" | "rem" => {
                debug!("builtin_op: {} % {}", x, y);
                apply_binop!(rem, x, y)
            }
            "^" | "pow" => {
                debug!("builtin_op: Raise {} to the {} power", x, y);
                let y_num = y.as_num()?;
                let x_num = x.as_num()?;
                let mut coll = 1;
                for _ in 0..y_num {
                    coll *= x_num;
                }
                x = lval_num(coll);
            }
            "min" => {
                debug!("builtin_op: Min {} and {}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num < y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            "max" => {
                debug!("builtin_op: Max {} and {}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num > y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            _ => unreachable!(),
        }
    }
    Ok(x)
}

// Operator aliases, function pointers will be stored in env
pub fn builtin_add(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "+")
}

pub fn builtin_sub(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "-")
}

pub fn builtin_mul(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "*")
}

pub fn builtin_div(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "/")
}

pub fn builtin_pow(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "^")
}

pub fn builtin_rem(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "%")
}

pub fn builtin_max(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "max")
}

pub fn builtin_min(_e: LenvT, a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "min")
}

// Attach a value to the front of a qexpr
pub fn builtin_cons(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let child_count = v.len()?;
    if child_count != 2 {
        return Err(BlisprError::NumArguments(2, child_count));
    }
    let new_elem = lval_pop(&mut v, 0)?;
    let qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut ret = lval_qexpr();
            lval_add(&mut ret, new_elem)?;
            for c in children {
                lval_add(&mut ret, c.clone())?;
            }
            Ok(ret)
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", v),
        )),
    }
}

// Evaluate qexpr as a sexpr
pub fn builtin_eval(e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut new_sexpr = lval_sexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_sexpr, cloned)?;
            }
            debug!("builtin_eval: {:?}", new_sexpr);
            lval_eval(e, new_sexpr)
        }
        _ => {
            // add it back
            lval_add(&mut v, qexpr)?;
            lval_eval(e, v)
        }
    }
}

// Return the first element of a qexpr
pub fn builtin_head(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let mut qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref mut children) => {
            if children.is_empty() {
                return Err(BlisprError::EmptyList);
            }
            debug!(
                "builtin_head: Returning the first element of {:?}",
                children
            );
            Ok(children[0].clone())
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", qexpr),
        )),
    }
}

// Return everything but the last element of a qexpr
pub fn builtin_init(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut ret = lval_qexpr();
            for item in children.iter().take(children.len() - 1) {
                lval_add(&mut ret, item.clone())?;
            }
            Ok(ret)
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", qexpr),
        )),
    }
}

// Join the children into one qexpr
pub fn builtin_join(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let mut ret = lval_qexpr();
    for _ in 0..v.len()? {
        let next = lval_pop(&mut v, 0)?;
        match *next {
            Lval::Qexpr(_) => {
                lval_join(&mut ret, next)?;
            }
            _ => {
                return Err(BlisprError::WrongType(
                    "qexpr".to_string(),
                    format!("{:?}", next),
                ))
            }
        }
    }
    Ok(ret)
}

// make sexpr into a qexpr
pub fn builtin_list(_e: LenvT, v: Box<Lval>) -> BlisprResult {
    match *v {
        Lval::Sexpr(ref children) => {
            debug!("builtin_list: Building list from {:?}", children);
            let mut new_qexpr = lval_qexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_qexpr, cloned)?;
            }
            Ok(new_qexpr)
        }
        _ => Ok(v),
    }
}

pub fn builtin_len(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let child_count = v.len()?;
    match child_count {
        1 => {
            let qexpr = lval_pop(&mut v, 0)?;
            match *qexpr {
                Lval::Qexpr(_) => {
                    debug!("Returning length of {}", qexpr);
                    Ok(lval_num(qexpr.len()? as i64))
                }
                _ => Err(BlisprError::WrongType(
                    "qexpr".to_string(),
                    format!("{:?}", qexpr),
                )),
            }
        }
        _ => Err(BlisprError::NumArguments(1, child_count)),
    }
}

pub fn builtin_tail(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let mut qexpr = lval_pop(&mut v, 0)?;
    debug!("Returning tail of {}", qexpr);
    match *qexpr {
        Lval::Qexpr(ref mut children) => {
            if children.is_empty() {
                return Err(BlisprError::EmptyList);
            }
            let mut ret = lval_qexpr();
            for c in &children[1..] {
                lval_add(&mut ret, c.clone())?;
            }
            Ok(ret)
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", qexpr),
        )),
    }
}

pub fn lval_eval(e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let child_count;
    match *v {
        Lval::Sym(s) => {
            let r = e.read().unwrap();
            return Ok(r.get(&s)?);
        }
        Lval::Sexpr(ref mut cells) => {
            debug!("lval_eval: Sexpr, evaluating children");
            // First, evaluate all the cells inside
            child_count = cells.len();
            for item in cells.iter_mut().take(child_count) {
                *item = lval_eval(Arc::clone(&e), item.clone())?
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
        lval_pop(&mut v, 0)
    } else {
        // Function call
        // Ensure the first element is a Symbol
        let fp = lval_pop(&mut v, 0)?;
        debug!("Calling function {:?} on {:?}", fp, v);
        match *fp {
            Lval::Fun(f) => f(Arc::clone(&e), v),
            _ => {
                println!("{}", *fp);
                Err(BlisprError::UnknownFunction(format!("{}", fp)))
            }
        }
    }
}
