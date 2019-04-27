use crate::{
    error::BlisprError,
    lval::{lval_add, lval_join, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval},
};
use std::ops::{Add, Div, Mul, Rem, Sub};

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

fn builtin_op<'a>(mut v: Box<Lval<'a>>, func: &str) -> Result<Box<Lval<'a>>, BlisprError> {
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
        debug!("Unary negation on {}", x);
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
                    return Err(BlisprError::DivideByZero);
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
fn builtin_eval<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
    let qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut new_sexpr = lval_sexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_sexpr, cloned)?;
            }
            debug!("builtin_eval: {:?}", new_sexpr);
            lval_eval(new_sexpr)
        }
        _ => {
            debug!("Failed builtin_eval on {:?}", qexpr);
            Err(BlisprError::WrongType(
                "qexpr".to_string(),
                format!("{:?}", qexpr),
            ))
        }
    }
}

// Return the first element of a qexpr
fn builtin_head<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
    let mut qexpr = lval_pop(&mut v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref mut children) => {
            if children.is_empty() {
                return Err(BlisprError::EmptyList);
            }
            Ok(children[0].clone())
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", qexpr),
        )),
    }
}

// Join the children into one qexpr
fn builtin_join<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
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
fn builtin_list<'a>(v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
    match *v {
        Lval::Sexpr(ref children) => {
            debug!("Building list from {:?}", children);
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

fn builtin_len<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
    let child_count = v.len()?;
    match child_count {
        1 => {
            let qexpr = lval_pop(&mut v, 0)?;
            match *qexpr {
                Lval::Qexpr(_) => Ok(lval_num(qexpr.len()? as i64)),
                _ => Err(BlisprError::WrongType(
                    "qexpr".to_string(),
                    format!("{:?}", qexpr),
                )),
            }
        }
        _ => Err(BlisprError::NumArguments(1, child_count)),
    }
}

fn builtin_tail<'a>(mut v: Box<Lval<'a>>) -> Result<Box<Lval<'a>>, BlisprError> {
    let mut qexpr = lval_pop(&mut v, 0)?;
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

fn builtin<'a>(v: Box<Lval<'a>>, func: &str) -> Result<Box<Lval<'a>>, BlisprError> {
    match func {
        "+" | "-" | "*" | "/" | "%" | "^" | "add" | "sub" | "mul" | "div" | "rem" | "pow"
        | "max" | "min" => builtin_op(v, func),
        "eval" => builtin_eval(v),
        "head" => builtin_head(v),
        "join" => builtin_join(v),
        "len" => builtin_len(v),
        "list" => builtin_list(v),
        "tail" => builtin_tail(v),
        _ => Err(BlisprError::UnknownFunction("func".to_string())),
    }
}

pub fn lval_eval(mut v: Box<Lval>) -> Result<Box<Lval>, BlisprError> {
    let child_count;
    match *v {
        Lval::Sexpr(ref mut cells) => {
            debug!("lval_eval: Sexpr({:?})", cells);
            // First, evaluate all the cells inside
            child_count = cells.len();
            for item in cells.iter_mut().take(child_count) {
                *item = lval_eval(item.clone())?
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
        let lfn = lval_pop(&mut v, 0)?;
        debug!("Calling function {} on {:?}", lfn, v);
        match *lfn {
            Lval::Sym(s) => builtin(v, &s),
            _ => {
                println!("{}", *lfn);
                Err(BlisprError::WrongType(
                    "symbol".to_string(),
                    format!("{:?}", lfn),
                ))
            }
        }
    }
}
