// the book uses two arrays
// I don't see any reason not to use a HashMap
// Will be interesting to benchmark later
use crate::{
    error::{BlisprError, BlisprResult},
    eval::*,
    lval::{lval_fun, LBuiltin, Lval},
};
use hashbrown::HashMap;

#[derive(PartialEq)]
pub struct Lenv<'a> {
    lookup: HashMap<String, Box<Lval<'a>>>,
}

impl<'a> Lenv<'a> {
    pub fn new() -> Self {
        let mut ret = Self {
            lookup: HashMap::new(),
        };

        ret.add_builtin("cons", builtin_cons);
        ret.add_builtin("eval", builtin_eval);
        ret.add_builtin("head", builtin_head);
        ret.add_builtin("init", builtin_init);
        ret.add_builtin("list", builtin_list);
        ret.add_builtin("join", builtin_join);
        ret.add_builtin("len", builtin_len);
        ret.add_builtin("tail", builtin_tail);

        ret.add_builtin("+", builtin_add);
        ret.add_builtin("add", builtin_add);
        ret.add_builtin("-", builtin_sub);
        ret.add_builtin("sub", builtin_sub);
        ret.add_builtin("*", builtin_mul);
        ret.add_builtin("mul", builtin_mul);
        ret.add_builtin("/", builtin_div);
        ret.add_builtin("div", builtin_div);
        ret.add_builtin("^", builtin_pow);
        ret.add_builtin("pow", builtin_pow);
        ret.add_builtin("%", builtin_rem);
        ret.add_builtin("rem", builtin_rem);
        ret.add_builtin("min", builtin_min);
        ret.add_builtin("max", builtin_max);

        ret
    }

    fn add_builtin(&mut self, name: &str, func: LBuiltin<'a>) {
        self.put(name, lval_fun(func));
    }

    pub fn get(&self, k: &str) -> BlisprResult<'a> {
        match self.lookup.get(k) {
            Some(v) => Ok(Box::new(*v.clone())),
            None => Err(BlisprError::UnknownFunction(k.to_string())),
        }
    }

    pub fn put(&mut self, k: &str, v: Box<Lval<'a>>) {
        let current = self.lookup.entry(k.into()).or_insert_with(|| v.clone());
        if *v != **current {
            // if it already existed, overwrite it with v
            *current = v;
        }
    }
}
