use crate::{
    error::{BlisprError, BlisprResult},
    lenv::{LenvT, ENV},
    lval::{
        lval_add, lval_join, lval_lambda, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval, LvalFun,
    },
};
use std::{
    ops::{Add, Div, Mul, Rem, Sub},
    sync::{Arc, RwLock},
};

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

// apply a binary operation {+ - * / ^ % min max} to a list of arguments in succession
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

// define a list of values
// if "def" define in global env
// if "=" define in local env
fn builtin_var(mut a: Box<Lval>, func: &str) -> BlisprResult {
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
            debug!("builtin_var: names: {:?} | vals: {:?}", names, vals);
            // TODO assert all symbols?
            if vals_len != names_len {
                Err(BlisprError::NumArguments(names_len, vals_len))
            } else {
                // grab write lock on ENV
                let mut w = ENV.write()?;
                for (k, v) in names.iter().zip(vals.iter()) {
                    let scope = if func == "def" { "global" } else { "local" };
                    debug!("adding key, value pair {:?}, {:?} to {} env", k, v, scope);
                    let name = k.clone().as_string()?;
                    if scope == "local" {
                        w.put(name, v.clone());
                    } else {
                        w.def(name, v.clone())?;
                    }
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

pub fn builtin_def(e: LenvT, v: Box<Lval>) -> BlisprResult {
    builtin_var(v, "def")
}

pub fn builtin_put(e: LenvT, v: Box<Lval>) -> BlisprResult {
    builtin_var(v, "=")
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

// terminate the program (or exit the prompt)
pub fn builtin_exit(_e: LenvT, _v: Box<Lval>) -> BlisprResult {
    // always succeeds
    println!("Goodbye!");
    ::std::process::exit(0);
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

//builtin_lambda returns a lambda lval from two lists of symbols
pub fn builtin_lambda(_e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    // ensure there's only two arguments
    let child_count = v.len()?;
    if child_count != 2 {
        return Err(BlisprError::NumArguments(2, child_count));
    }

    // first qexpr should contain only symbols - lval.as_string().is_ok()
    let formals = lval_pop(&mut v, 0)?;
    let formals_ret = formals.clone(); // ewwww but it gets moved on me?!
    let body = lval_pop(&mut v, 0)?;
    match *formals {
        Lval::Qexpr(contents) => {
            for cell in contents {
                if cell.as_string().is_err() {
                    return Err(BlisprError::WrongType(
                        "Symbol".to_string(),
                        format!("{:?}", cell),
                    ));
                }
            }
            match *body {
                Lval::Qexpr(_) => Ok(lval_lambda(formals_ret, body)),
                _ => Err(BlisprError::WrongType(
                    "Q-Expression".to_string(),
                    format!("{:?}", body),
                )),
            }
        }
        _ => Err(BlisprError::WrongType(
            "Q-Expression".to_string(),
            format!("{:?}", formals),
        )),
    }
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

// Print all the named variables in the environment
pub fn builtin_printenv(e: LenvT, _v: Box<Lval>) -> BlisprResult {
    // we don't use the input
    lval_eval(e, e.list_all()?)
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

// Call a Lval::Fun(f) on an argument list
// This will handle both builtins and lambdas
pub fn lval_call(e: LenvT, f: Box<Lval>, mut args: Box<Lval>) -> BlisprResult {
    match *f {
        Lval::Fun(func) => {
            match func {
                // If it's a builtin, just call the function pointer
                LvalFun::Builtin(fp) => fp(e, args),
                LvalFun::Lambda(mut env, mut formals, body) => {
                    debug!(
                        "Executing lambda.  Formals: {:?}, body: {:?}",
                        formals, body
                    );
                    // If it's a Lambda, bind arguments to env
                    // first grab the argument and body
                    let given = args.len()?;
                    let total = formals.len()?;

                    while args.len()? > 0 {
                        // if we've run out of args to bind, error
                        if formals.len()? == 0 {
                            return Err(BlisprError::NumArguments(total, given));
                        }

                        // grab first symbol from formals
                        let sym = lval_pop(&mut formals, 0)?;

                        // grab next argument from list
                        let val = lval_pop(&mut args, 0)?;

                        // bind a copy to the function's environment
                        env.put(sym.as_string()?, val);
                    }
                    // if all formals have been bound
                    if formals.len()? == 0 {
                        // set environment parent to evaluation environment
                        // TODO is this fucked?  Do I need to be passing it in?
                        // The book is passing the env around as an arg
                        // Im worried I'll need to do that so that each lval_*() fn has the proper env
                        // This worked fine for global, not sure it will recur
                        env.parent = Arc::clone(&e);

                        // Evaluate and return
                        let mut ret = lval_sexpr();
                        lval_add(&mut ret, body)?;
                        debug!("Evaluating fully applied lambda");
                        builtin_eval(e, ret)
                    } else {
                        // Otherwise return partially evaluated function
                        // build a new lval for it
                        debug!("Returning partially applied lambda");
                        Ok(lval_lambda(formals, body))
                    }
                }
            }
        }
        _ => Err(BlisprError::WrongType(
            "Function".to_string(),
            format!("{:?}", f),
        )),
    }
}

// Fully evaluate an `Lval`
pub fn lval_eval(e: LenvT, mut v: Box<Lval>) -> BlisprResult {
    let child_count;
    match *v {
        Lval::Sym(s) => {
            // If it's a symbol, perform an environment lookup
            let result = e.get(&s)?;
            debug!(
                "lval_eval: Symbol lookup - retrieved {:?} from key {}",
                result, s
            );
            // The environment stores Lvals ready to go, we're done
            return Ok(result);
        }
        Lval::Sexpr(ref mut cells) => {
            // If it's a Sexpr, we're going to continue past this match
            // First, though, recursively evaluate each child with lval_eval()
            debug!("lval_eval: Sexpr, evaluating children");
            child_count = cells.len();
            for item in cells.iter_mut().take(child_count) {
                *item = lval_eval(e, item.clone())?
            }
        }
        // if it's not a sexpr, we're done, return as is
        _ => {
            debug!("lval_eval: Non-sexpr: {:?}", v);
            return Ok(v);
        }
    }

    // Anything other than an Lval will have already been returned
    // Handle the different Sexpr cases

    if child_count == 0 {
        // It was a Sexpr, but it was empty.  We're done, return it
        Ok(v)
    } else if child_count == 1 {
        // Single expression
        debug!("Single-expression");
        lval_eval(e, lval_pop(&mut v, 0)?)
    } else {
        // Function call
        // We'll pop the first element off and attempt to call it on the rest of the elements
        // lval_call will handle typechecking fp
        let fp = lval_pop(&mut v, 0)?;
        debug!("Calling function {:?} on {:?}", fp, v);
        lval_call(e, fp, v)
    }
}
