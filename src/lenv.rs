// the book uses two arrays
// I don't see any reason not to use a HashMap
// Will be interesting to benchmark later
use crate::{
    error::{BlisprError, BlisprResult, Result},
    eval::*,
    lval::{lval_add, lval_builtin, lval_qexpr, lval_sym, LBuiltin, Lval},
};
use hashbrown::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Lenv<'a> {
    lookup: HashMap<String, Box<Lval>>,
    pub parent: Option<&'a mut Lenv<'a>>,
}

impl<'a> Lenv<'a> {
    pub fn new(parent: Option<&mut Lenv>) -> Self {
        let mut ret = Self {
            lookup: HashMap::new(),
            parent,
        };

        // Register builtins

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
                env.def(k, v)?;
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
                    Some(p_env) => p_env.get(k),
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

impl<'a> fmt::Display for Lenv<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parent_str = if let Some(_) = self.parent {
            "Child"
        } else {
            "Root"
        };
        write!(f, "{} vals in env | {}", self.lookup.len(), parent_str)
    }
}

impl<'a> PartialEq for Lenv<'a> {
    fn eq(&self, other: &Lenv) -> bool {
        // Note - doesn't compare parents!
        self.lookup == other.lookup
    }
}
