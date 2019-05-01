// the book uses two arrays
// I don't see any reason not to use a HashMap
// Will be interesting to benchmark later
use crate::{
    error::{BlisprError, BlisprResult, Result},
    eval::*,
    lval::{lval_add, lval_builtin, lval_qexpr, lval_sym, LBuiltin, Lval},
};
use hashbrown::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref ENV: LenvT = new_lenvt();
}

pub type LenvT = Arc<RwLock<Lenv>>;

#[derive(Debug, Clone)]
pub struct Lenv {
    lookup: HashMap<String, Box<Lval>>,
    pub parent: Option<LenvT>,
}

impl Lenv {
    pub fn new(parent: Option<LenvT>) -> Self {
        let mut ret = Self {
            lookup: HashMap::new(),
            parent,
        };

        // Definiton
        ret.add_builtin("\\", builtin_lambda);
        ret.add_builtin("def", builtin_def);
        ret.add_builtin("=", builtin_put);

        // List manipulation
        ret.add_builtin("cons", builtin_cons);
        ret.add_builtin("eval", builtin_eval);
        ret.add_builtin("head", builtin_head);
        ret.add_builtin("init", builtin_init);
        ret.add_builtin("list", builtin_list);
        ret.add_builtin("join", builtin_join);
        ret.add_builtin("len", builtin_len);
        ret.add_builtin("tail", builtin_tail);

        // Utility
        ret.add_builtin("exit", builtin_exit);
        ret.add_builtin("printenv", builtin_printenv);

        // Arithmetic
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

    // register a function pointer to the global scope
    fn add_builtin(&mut self, name: &str, func: LBuiltin) {
        self.put(name.to_string(), lval_builtin(func));
    }

    // add a function to the global scope
    pub fn def(&mut self, k: String, v: Box<Lval>) -> Result<()> {
        // iterate up through parents until we find the root
        match &self.parent {
            Some(env) => {
                env.write()?.def(k, v)?;
                Ok(())
            }
            None => {
                self.put(k, v);
                Ok(())
            }
        }
    }

    // retrieve a value from the env, local first then up through parents
    pub fn get(&self, k: &str) -> BlisprResult {
        match self.lookup.get(k) {
            Some(v) => Ok(Box::new(*v.clone())),
            None => {
                // if we didn't find it in self, check the parent
                // this will recur all the way up to the global scope
                match &self.parent {
                    None => Err(BlisprError::UnknownFunction(k.to_string())),
                    Some(p_env) => {
                        let arc = Arc::clone(&p_env);
                        let r = arc.read()?;
                        r.get(k)
                    }
                }
            }
        }
    }

    // Returns an Lval containing Symbols with each k,v pair in the local env
    pub fn list_all(&self) -> BlisprResult {
        let mut ret = lval_qexpr();
        for (k, v) in &self.lookup {
            lval_add(&mut ret, lval_sym(&format!("{}:{}", k, v)))?;
        }
        Ok(ret)
    }

    // add a value to the local env
    pub fn put(&mut self, k: String, v: Box<Lval>) {
        let current = self.lookup.entry(k).or_insert_with(|| v.clone());
        if *v != **current {
            // if it already existed, overwrite it with v
            *current = v;
        }
    }
}

impl PartialEq for Lenv {
    fn eq(&self, other: &Lenv) -> bool {
        let parent_lookup = match &self.parent {
            Some(arc) => arc.read().unwrap(),
            _ => return true,
        };
        let other_parent_lookup = match &other.parent {
            Some(arc) => arc.read().unwrap(),
            _ => return true,
        };
        self.lookup == other.lookup && *parent_lookup == *other_parent_lookup
    }
}

pub fn new_lenvt() -> LenvT {
    Arc::new(RwLock::new(Lenv::new(None)))
}
