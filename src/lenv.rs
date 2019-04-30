// the book uses two arrays
// I don't see any reason not to use a HashMap
// Will be interesting to benchmark later
use crate::{
    error::{BlisprError, BlisprResult},
    eval::*,
    lval::{lval_add, lval_builtin, lval_qexpr, lval_sym, LBuiltin, Lval},
};
use hashbrown::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref ENV: LenvT = Arc::new(RwLock::new(Lenv::new()));
}

pub type LenvT = Arc<RwLock<Lenv>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Lenv {
    lookup: HashMap<String, Box<Lval>>,
}

impl Lenv {
    pub fn new() -> Self {
        let mut ret = Self {
            lookup: HashMap::new(),
        };

        ret.add_builtin("def", builtin_def);
        ret.add_builtin("cons", builtin_cons);
        ret.add_builtin("eval", builtin_eval);
        ret.add_builtin("exit", builtin_exit);
        ret.add_builtin("head", builtin_head);
        ret.add_builtin("init", builtin_init);
        ret.add_builtin("lambda", builtin_lambda);
        ret.add_builtin("list", builtin_list);
        ret.add_builtin("join", builtin_join);
        ret.add_builtin("printenv", builtin_printenv);
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

    fn add_builtin(&mut self, name: &str, func: LBuiltin) {
        self.put(name.to_string(), lval_builtin(func));
    }

    pub fn get(&self, k: &str) -> BlisprResult {
        match self.lookup.get(k) {
            Some(v) => Ok(Box::new(*v.clone())),
            None => Err(BlisprError::UnknownFunction(k.to_string())),
        }
    }

    pub fn list_all(&self) -> BlisprResult {
        let mut ret = lval_qexpr();
        for (k, v) in &self.lookup {
            lval_add(&mut ret, lval_sym(&format!("{}:{}", k, v)))?;
        }
        Ok(ret)
    }

    pub fn put(&mut self, k: String, v: Box<Lval>) {
        let current = self.lookup.entry(k).or_insert_with(|| v.clone());
        if *v != **current {
            // if it already existed, overwrite it with v
            *current = v;
        }
    }
}
