use crate::{
    error::{BlisprError, BlisprResult},
    lenv::Lenv,
    lval::{
        lval_add, lval_join, lval_lambda, lval_num, lval_pop, lval_qexpr, lval_sexpr, Lval, LvalFun,
    },
};
use log::debug;
use std::{collections::HashMap, ops::{Add, Div, Mul, Rem, Sub}};

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
fn builtin_op(mut v: &mut Lval, func: &str) -> BlisprResult {
    let mut child_count;
    match *v {
        Lval::Sexpr(ref children) => {
            child_count = children.len();
        }
        _ => return Ok(Box::new(v.clone())),
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
// TODO macro??  create_builtin!(a, &str)
pub fn builtin_add(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "+")
}

pub fn builtin_sub(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "-")
}

pub fn builtin_mul(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "*")
}

pub fn builtin_div(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "/")
}

pub fn builtin_pow(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "^")
}

pub fn builtin_rem(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "%")
}

pub fn builtin_max(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "max")
}

pub fn builtin_min(a: &mut Lval) -> BlisprResult {
    builtin_op(a, "min")
}

// define a list of values
// if "def" define in global env
// if "=" define in local env
fn builtin_var(e: &mut Lenv, a: &mut Lval, func: &str) -> BlisprResult {
    let args = lval_pop(a, 0)?;
    match *args {
        Lval::Qexpr(names) => {
            // grab the rest of the vals
            let mut vals = Vec::new();
            for _ in 0..a.len()? {
                vals.push(lval_pop(a, 0)?);
            }
            let names_len = names.len();
            let vals_len = vals.len();
            // TODO assert all symbols?
            if vals_len != names_len {
                Err(BlisprError::NumArguments(names_len, vals_len))
            } else {
                for (k, v) in names.iter().zip(vals.iter()) {
                    let scope = if func == "def" { "global" } else { "local" };
                    debug!("adding key, value pair {}, {} to {} env {}", k, v, scope, e);
                    let name = k.clone().as_string()?;
                    if scope == "local" {
                        e.put(name, v.clone());
                    } else {
                        //e.def(name, v.clone())?;
                        debug!("warning: global scope definition unimplemented!");
                        e.put(name, v.clone());
                    }
                }
                Ok(lval_sexpr())
            }
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", args),
        )),
    }
}

// BROKEN
//pub fn builtin_def_stub(_v: &Lval) -> BlisprResult {
//    Ok(lval_sexpr())
//}

// FOR NOW def IS LOCAL ENV ASSIGN
fn builtin_def(e: &mut Lenv, v: &mut Lval) -> BlisprResult {
    builtin_var(e, v, "def")
}

pub fn builtin_put_stub(_v: &mut Lval) -> BlisprResult {
    Ok(lval_sexpr())
}

//BROKEN
//fn builtin_put(e: &mut Lenv, v: &Lval) -> BlisprResult {
//    builtin_var(e, v, "=")
//}

// Attach a value to the front of a qexpr
pub fn builtin_cons(v: &mut Lval) -> BlisprResult {
    let child_count = v.len()?;
    if child_count != 2 {
        return Err(BlisprError::NumArguments(2, child_count));
    }
    let new_elem = lval_pop(v, 0)?;
    let qexpr = lval_pop(v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut ret = lval_qexpr();
            lval_add(&mut ret, &new_elem)?;
            for c in children {
                lval_add(&mut ret, &c.clone())?;
            }
            Ok(ret)
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", v),
        )),
    }
}

// correct call dispatched in lval_call
pub fn builtin_eval_stub(_v: &mut Lval) -> BlisprResult {
    Ok(lval_sexpr())
}

// Evaluate qexpr as a sexpr
pub fn builtin_eval(e: &mut Lenv, v: &mut Lval) -> BlisprResult {
    let qexpr = lval_pop(v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut new_sexpr = lval_sexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_sexpr, &cloned)?;
            }
            debug!("builtin_eval: {:?}", new_sexpr);
            lval_eval(e, &mut new_sexpr)
        }
        _ => {
            // add it back
            lval_add(v, &qexpr)?;
            lval_eval(e, v)
        }
    }
}

// terminate the program (or exit the prompt)
pub fn builtin_exit(_v: &mut Lval) -> BlisprResult {
    // always succeeds
    println!("Goodbye!");
    ::std::process::exit(0);
}

// Return the first element of a qexpr
pub fn builtin_head(v: &mut Lval) -> BlisprResult {
    let mut qexpr = lval_pop(v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref mut children) => {
            if children.is_empty() {
                return Err(BlisprError::EmptyList);
            }
            debug!("builtin_head: Returning the first element");
            Ok(children[0].clone())
        }
        _ => Err(BlisprError::WrongType(
            "qexpr".to_string(),
            format!("{:?}", qexpr),
        )),
    }
}

// Return everything but the last element of a qexpr
pub fn builtin_init(v: &mut Lval) -> BlisprResult {
    let qexpr = lval_pop(v, 0)?;
    match *qexpr {
        Lval::Qexpr(ref children) => {
            let mut ret = lval_qexpr();
            for item in children.iter().take(children.len() - 1) {
                lval_add(&mut ret, &item.clone())?;
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
pub fn builtin_join(v: &mut Lval) -> BlisprResult {
    let mut ret = lval_qexpr();
    for _ in 0..v.len()? {
        let next = lval_pop(v, 0)?;
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
pub fn builtin_lambda(v: &mut Lval) -> BlisprResult {
    // ensure there's only two arguments
    let child_count = v.len()?;
    if child_count != 2 {
        return Err(BlisprError::NumArguments(2, child_count));
    }

    // first qexpr should contain only symbols - lval.as_string().is_ok()
    let formals = lval_pop(v, 0)?;
    let formals_ret = formals.clone(); // ewwww but it gets moved on me?!  this might be why Rc<> - it doesn't need to mutate
    let body = lval_pop(v, 0)?;
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
                Lval::Qexpr(_) => Ok(lval_lambda(HashMap::new(), formals_ret, body)),
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
pub fn builtin_list(v: &mut Lval) -> BlisprResult {
    match *v {
        Lval::Sexpr(ref children) => {
            debug!("builtin_list: Building qexpr from {:?}", children);
            let mut new_qexpr = lval_qexpr();
            for c in children {
                let cloned = Box::new(*c.clone());
                lval_add(&mut new_qexpr, &cloned)?;
            }
            Ok(new_qexpr)
        }
        _ => Ok(Box::new(v.clone())),
    }
}

pub fn builtin_len(v: &mut Lval) -> BlisprResult {
    let child_count = v.len()?;
    match child_count {
        1 => {
            let qexpr = lval_pop(v, 0)?;
            match *qexpr {
                Lval::Qexpr(_) => {
                    debug!("Returning length of {:?}", qexpr);
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

pub fn builtin_printenv_stub(_v: &mut Lval) -> BlisprResult {
    Ok(lval_sexpr())
}

// Print all the named variables in the environment
pub fn builtin_printenv(e: &mut Lenv) -> BlisprResult {
    // we don't use the input
    lval_eval(e, &mut *e.list_all()?)
}

pub fn builtin_tail(v: &mut Lval) -> BlisprResult {
    let mut qexpr = lval_pop(v, 0)?;
    debug!("Returning tail of {:?}", qexpr);
    match *qexpr {
        Lval::Qexpr(ref mut children) => {
            if children.is_empty() {
                return Err(BlisprError::EmptyList);
            }
            let mut ret = lval_qexpr();
            for c in &children[1..] {
                lval_add(&mut ret, &c.clone())?;
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
pub fn lval_call(e: &mut Lenv, f: Lval, args: &mut Lval) -> BlisprResult {
    match f {
        Lval::Fun(func) => {
            match func {
                // if its one of the ones that need an environment, intercept and route to the properly typed fn
                LvalFun::Builtin(name, fp) => match name.as_str() {
                    "eval" => builtin_eval(e, args),
                    "def" => builtin_def(e, args),
                    //"=" => builtin_put(e, args),
                    "printenv" => builtin_printenv(e),
                    // Otherwise, just apply the actual stored function pointer
                    _ => fp(args),
                },
                LvalFun::Lambda(env, mut formals, body) => {
                    debug!(
                        "Executing lambda.  Environment: {:?}, Formals: {:?}, body: {:?}",
                        env, formals, body
                    );
                    // If it's a Lambda, bind arguments to a new local environment

                    // First, build the lookup hashmap
                    let mut new_env: HashMap<String, Box<Lval>> = HashMap::new();
                    // grab the argument and body
                    let given = args.len()?;
                    let total = formals.len()?;

                    while args.len()? > 0 {
                        // if we've run out of args to bind, error
                        if formals.len()? == 0 {
                            return Err(BlisprError::NumArguments(total, given));
                        }

                        // grab first symbol from formals
                        let sym = lval_pop(&mut formals, 0)?;

                        // special case to handle '&'
                        if &sym.as_string()? == "&" {
                            // make sure there's one symbol left
                            if formals.len()? != 1 {
                                return Err(BlisprError::FunctionFormat);
                            }

                            // next formal should be found to remaining args
                            let next_sym = lval_pop(&mut formals, 0)?;
                            let arglist = builtin_list(args)?;
                            let curr = new_env
                                .entry(next_sym.as_string()?)
                                .or_insert(arglist.clone());
                            if *curr != arglist {
                                *curr = arglist.clone();
                            }
                            break;
                        }

                        // grab next argument from list
                        let val = lval_pop(args, 0)?;

                        // bind a copy to the function's environment
                        debug!("lval_call: adding {},{} to local fn environment", sym, val);
                        let curr = new_env.entry(sym.as_string()?).or_insert(val.clone());
                        // if we're overwriting, overwrite!
                        if *curr != val {
                            *curr = val.clone();
                        }
                    }
                    // Use the lookup map to initialize the new child env for evaluation
                    let mut local_env = Lenv::new(Some(new_env.clone()), Some(e));
                    // if all formals have been bound
                    if formals.len()? == 0 {
                        // Evaluate and return
                        // first, apply any held by the lambda.
                        for (k, v) in env {
                            local_env.put(k, v);
                        }
                        let mut ret = lval_sexpr();
                        lval_add(&mut ret, &body)?;
                        debug!("lval_call: evaluating fully applied lambda {}", ret);
                        // evaluate with the environment of the function, which now has the env this was called with as a parent.
                        builtin_eval(&mut local_env, &mut ret)
                    } else {
                        // Otherwise return partially evaluated function
                        // build a new lval for it
                        debug!("Returning partially applied lambda");
                        Ok(lval_lambda(new_env, formals.clone(), body.clone()))
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

// Given a slice of boxed Lvals, return a single evaluated sexpr
fn eval_cells(e: &mut Lenv, cells: &[Box<Lval>]) -> BlisprResult {
    cells.iter().fold(Ok(lval_sexpr()), |acc, c| {
        match acc {
            Ok(mut lval) => {
                lval_add(&mut lval, &*lval_eval(e, &mut c.clone())?)?;
                Ok(lval)
            }
            // it's just a Result so we can bubble errors out of the fold
            Err(_) => unreachable!(),
        }
    })
}

// Fully evaluate an `Lval`
pub fn lval_eval(e: &mut Lenv, v: &mut Lval) -> BlisprResult {
    let child_count;
    let mut args_eval;
    match v {
        Lval::Blispr(forms) => {
            // If it's multiple, evaluate each and return the result of the last
            args_eval = eval_cells(e, forms)?;
            let forms_len = args_eval.len()?;
            return Ok(lval_pop(&mut args_eval, forms_len - 1)?);
        }
        Lval::Sym(s) => {
            // If it's a symbol, perform an environment lookup
            let result = e.get(&s)?;
            debug!(
                "lval_eval: Symbol lookup - retrieved {:?} from key {:?}",
                result, s
            );
            // The environment stores Lvals ready to go, we're done
            return Ok(result);
        }
        Lval::Sexpr(ref mut cells) => {
            // If it's a Sexpr, we're going to continue past this match
            // First, though, recursively evaluate each child with lval_eval()
            debug!("lval_eval: Sexpr, evaluating children");
            // grab the length and evaluate the children
            child_count = cells.len();
            args_eval = eval_cells(e, cells)?;
        }
        // if it's not a sexpr, we're done, return as is
        _ => {
            debug!("lval_eval: Non-sexpr: {:?}", v);
            return Ok(Box::new(v.clone()));
        }
    }
    if child_count == 0 {
        // It was a Sexpr, but it was empty.  We're done, return it
        Ok(Box::new(v.clone()))
    } else if child_count == 1 {
        // Single expression
        debug!("Single-expression");
        lval_eval(e, &mut *lval_pop(v, 0)?)
    } else {
        // Function call
        // We'll pop the first element off and attempt to call it on the rest of the elements
        // lval_call will handle typechecking fp
        let fp = lval_pop(&mut args_eval, 0)?;
        debug!("Calling function {:?} on {:?}", fp, v);
        lval_call(e, *fp, &mut *args_eval)
    }
}
