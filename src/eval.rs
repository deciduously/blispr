use crate::{
    error::{BlisprError, BlisprResult},
    lenv::ENV,
    lval::{lval_add, lval_join, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval, LvalFun},
};
use std::ops::{Add, Div, Mul, Rem, Sub};

// macro to shorten code for applying a binary operation to two Lvals
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

// apply a binary operation: + - * / ^ % min max
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
                debug!("builtin_op: Add {:?} and {:?}", x, y);
                apply_binop!(add, x, y)
            }
            "-" | "sub" => {
                debug!("builtin_op: Subtract {:?} and {:?}", x, y);
                apply_binop!(sub, x, y)
            }
            "*" | "mul" => {
                debug!("builtin_op: Multiply {:?} and {:?}", x, y);
                apply_binop!(mul, x, y)
            }
            "/" | "div" => {
                if y.as_num()? == 0 {
                    debug!("builtin_op: Failed divide {:?} by {:?}", x, y);
                    return Err(BlisprError::DivideByZero);
                } else {
                    debug!("builtin_op: Divide {:?} by {:?}", x, y);
                    apply_binop!(div, x, y)
                }
            }
            "%" | "rem" => {
                debug!("builtin_op: {:?} % {:?}", x, y);
                apply_binop!(rem, x, y)
            }
            "^" | "pow" => {
                debug!("builtin_op: Raise {:?} to the {:?} power", x, y);
                let y_num = y.as_num()?;
                let x_num = x.as_num()?;
                let mut coll = 1;
                for _ in 0..y_num {
                    coll *= x_num;
                }
                x = lval_num(coll);
            }
            "min" => {
                debug!("builtin_op: Min {:?} and {:?}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num < y_num {
                    x = lval_num(x_num);
                } else {
                    x = lval_num(y_num);
                };
            }
            "max" => {
                debug!("builtin_op: Max {:?} and {:?}", x, y);
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
pub fn builtin_add(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "+")
}

pub fn builtin_sub(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "-")
}

pub fn builtin_mul(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "*")
}

pub fn builtin_div(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "/")
}

pub fn builtin_pow(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "^")
}

pub fn builtin_rem(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "%")
}

pub fn builtin_max(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "max")
}

pub fn builtin_min(a: Box<Lval>) -> BlisprResult {
    builtin_op(a, "min")
}

// define a list of values
pub fn builtin_def(mut a: Box<Lval>) -> BlisprResult {
    let args = lval_pop(&mut a, 0)?;
    match *args {
        Lval::Qexpr(names) => {
            // grab the rest of the vals
            let mut vals = Vec::new();
            for _ in 0..a.len()? {
                vals.push(lval_pop(&mut a, 0)?);
            }
            let names_len = names.len();
            let vals_len = vals.len();
            debug!("builtin_def: names: {:?} | vals: {:?}", names, vals);
            // TODO assert all symbols?
            if vals_len != names_len {
                Err(BlisprError::NumArguments(names_len, vals_len))
            } else {
                // grab write lock on ENV
                let mut w = ENV.write().unwrap();
                for (k, v) in names.iter().zip(vals.iter()) {
                    debug!("adding key, value pair {:?}, {:?} to env", k, v);
                    let name = k.clone().as_string()?;
                    w.put(name, v.clone());
                }
                Ok(lval_sexpr())

                // write lock dropped here
            }
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", args),
        )),
    }
}

// Attach a value to the front of a qexpr
pub fn builtin_cons(mut v: Box<Lval>) -> BlisprResult {
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
pub fn builtin_eval(mut v: Box<Lval>) -> BlisprResult {
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
            // add it back
            lval_add(&mut v, qexpr)?;
            lval_eval(v)
        }
    }
}

// terminate the program (or exit the prompt)
pub fn builtin_exit(_v: Box<Lval>) -> BlisprResult {
    // always succeeds
    println!("Goodbye!");
    ::std::process::exit(0);
}

// Return the first element of a qexpr
pub fn builtin_head(mut v: Box<Lval>) -> BlisprResult {
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
pub fn builtin_init(mut v: Box<Lval>) -> BlisprResult {
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
pub fn builtin_join(mut v: Box<Lval>) -> BlisprResult {
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
pub fn builtin_list(v: Box<Lval>) -> BlisprResult {
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

pub fn builtin_len(mut v: Box<Lval>) -> BlisprResult {
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

// Print all the named variables in the environment
pub fn builtin_printenv(_v: Box<Lval>) -> BlisprResult {
    // we don't use the input
    lval_eval(ENV.read().unwrap().list_all()?)
}

pub fn builtin_tail(mut v: Box<Lval>) -> BlisprResult {
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

pub fn lval_eval(mut v: Box<Lval>) -> BlisprResult {
    let child_count;
    match *v {
        Lval::Sym(s) => {
            let r = ENV.read().unwrap();
            return Ok(r.get(&s)?);
        }
        Lval::Sexpr(ref mut cells) => {
            debug!("lval_eval: Sexpr, evaluating children");
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
        let fp = lval_pop(&mut v, 0)?;
        debug!("Calling function {:?} on {:?}", fp, v);
        match *fp {
            Lval::Fun(lf) => match lf {
                LvalFun::Builtin(_, f) => f(v),
                _ => Err(BlisprError::WrongType(
                    "builtin".to_string(),
                    "lambda".to_string(),
                )),
            },
            _ => {
                println!("{}", *fp);
                Err(BlisprError::UnknownFunction(format!("{}", fp)))
            }
        }
    }
}
