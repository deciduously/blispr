# Rust Your Own Lisp

It runs out translating things by [orangeduck](http://theorangeduck.com/page/about) into [Rust](https://www.rust-lang.org/) is [fun](https://dev.to/deciduously/build-you-a-markov-chain-in-rust-or-whatever-54mo).  His book [Build Your Own Lisp](http://www.buildyourownlisp.com/) is fantastic, both as an introduction to C and an introduction to writing an interpreter, so here we go again!

This post is not intended to be a replacement for that text, by a long shot - go read the book.  It's excellent.  In translating to Rust, though, there are a few necessary differences worth noting.  This post does not include the code in its entirety but rather examples of each concept, and may be useful for anyone attempting a similar project or translation of their own in Rust.  I've also removed all debug logging for clarity.  The full implementation can be found in [this repo](https://github.com/deciduously/blispr).

## The Task

If you've never attempted something like this before, it's helpful to understand the high-level overview of the program we need to write.  This program will take a string as input and attempt to evaluate the result.  We need to *parse* the string into a tree of semantically tagged lexical tokens, *read* this parse tree of tokens into a structure called an [Abstract Syntax Tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (referred to as AST going forward), and then *evaluate* that AST.  To do this, we'll need to semantically tag each element so that our program can methodically work its way through, understanding what each part is.

Don't worry if you're not familiar with those data structures.  It's not as complicated as it sounds (and we're shelling parsing out to a library).  For a small concrete example, let's look at the input string `+ 2 (* 3 4) 5`.  To work with this input, we need to build a an AST structure like the following:

```rust
S-Expression(
    Symbol("+"),
    Number(2),
    S-Expression(
        Symbol("*"),
        Number(3),
        Number(4),
    ),
    Number (5),
)
```

The whole program is represented as an [S-Expression](https://en.wikipedia.org/wiki/S-expression).  When our program sees one of these with multiple elements, it's going to try to execute it as a function call, looking up the function from the symbol in the first position.  First, though, it's going to recursively evaluate all of its children - so if any of them are themselves s-expressions, we'll get them into values we can work with first.  In this example, the inner S-Expression `(* 3 4)` will be evaluated first:

```rust
S-Expression(
    Symbol("*"),
    Number(3),
    Number(4),
)
```

This will be interpreted as a 

```rust
S-Expression(
    Symbol("+"),
    Number(2),
    Number(12),
    Number(5),
)
```

Now we have a binary operation as the first element of the S-expression and two numbers with which to apply it.  When this evaluates, we're left with just `Number(14)`, which can be displayed to the user as the result of the computation.

As part of the evaluation of an S-Expression, we'll look for a function corresponding to the `Symbol` in the first position.  To do this we'll also be creating an environment where we can store key/value pairs - at the key `+` will be stored a function pointer to the function we need.  If the user tries to use a name that doesn't have a corresponding function it will return an error - `"unbound function"` or the likes.  User-defined functions will also create their own local environments, and any user-defined value created with, e.g. `def {x} 10` will be stored in this environment as well.

The other data type in this particular (idiosyncratic) lisp is `Qexpr`, and is represented with curly braces: `{1 2 3}`.  You can turn a list of arguments into a list with `list` and evaluate a list as if it were an S-Expression with `eval`:

```
lisp>{+ 1 2}
{+ 1 2}
lisp>list + 1 2
{+ 1 2}
lisp> eval (list + 1 2)
3
```

In addition to arithmetic operators, we'll add builtin functions to the environment to manipulate lists and define your own values and functions - you know, do programming!

I learned a lot about C, interpreters, and Rust from this project, and highly recommend the exercise.  For better or worse (probably worse), I've called this implementation `blispr`.  

## Rustyline

First thing's first, we've got to collect us some strings.  I highly recommend [`rustyline`](https://github.com/kkawakam/rustyline), a pure-Rust `readline` implementation.  This is all you have to do:

```rust
pub fn repl() -> Result<()> {
    println!("Blispr v0.0.1");
    println!("Use exit(), Ctrl-C, or Ctrl-D to exit prompt");

    let mut rl = Editor::<()>::new();
    if rl.load_history("./.blispr-history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");

        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                if let Err(e) = eval_str(&line) {
                    eprintln!("{}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                break;
            }
        }
    }
    rl.save_history("./.blispr-history.txt")?;
    Ok(())
}
```

One thing to note is that I'm not propagating the error that `eval_str` might throw up to the caller here with `?` - I don't want evaluation errors to crash the repl!  Anything that can happen inside `eval_str()` I just want to inform the user about with `eprintln!()` and loop again.

The next step is hinted at in the `Ok()` arm of the `match` - the meat of the work is happening in `eval_str()`:

```rust
pub fn eval_str(s: &str) -> Result<()> {
    let parsed = BlisprParser::parse(Rule::blispr, s)?.next().unwrap();
    let lval_ptr = lval_read(parsed)?;
    println!("{}", lval_eval(lval_ptr)?);
    Ok(())
}
```

This function does four things.  The first line stores the parse tree.  This tags our input string with semantic grammatical tags that we'll define below.  The next line reads that tree into our AST, which represents the whole program as a lisp value that can be evaluated recursively.  Finally, we print out the result of evaluating the AST.  Any errors that happened along the way were caught with the `?` operator - below we'll see what that `Result<T>` alias represents.

## Lval

To represent Abstract Syntax Tree, I used a Rust `enum` called `Lval`:

```rust
// LvalChildren is how the recursive types hold their children
type LvalChildren = Vec<Box<Lval>>;
// LBuiltin is an aliased function pointer
pub type LBuiltin = fn(Box<Lval>) -> BlisprResult;

// There are two types of functions
#[derive(Debug, Clone, PartialEq)]
pub enum LvalFun {
    Builtin(LBuiltin),                       // (function pointer)
    Lambda(Box<Lenv>, Box<Lval>, Box<Lval>), // (environment, formals, body), both should be Qexpr
}

// The main type - all possible Blispr values
#[derive(Debug, Clone, PartialEq)]
pub enum Lval {
    Fun(LvalFun),
    Num(i64),
    Sym(String),
    Sexpr(LvalChildren),
    Qexpr(LvalChildren),
}
```

Each variant carries its contents with it.  As we read our text, each element is going to be converted into the proper type of `Lval`.    In the simple sense, a string like `"4"` is going to be parsed into `Lval::Num(4)`.  Now our pogram will understand what this `4` shindig is all about.  I've also implemented [`fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) for this type, which is responsible for defining the output string to be finally displayed to the user.  With the auto-derived `Debug` trait we get something like `Lval::Num(4)`, and with `Display` we just get `4`.

We have numbers, symbols, functions (two different types of function - more on those later on), and two types of expression list - s-expressions and q-expressions.  S-expressions will be evaluated as code, looking for a function in the first position, and q-expressions are evaluated as just lists of data.  The whole program that's read in is going to be one big containing `Lval::Sexpr`, and we just need to evaluate it until we only have a result needing no futher evaluation, either a `Num`, `Sym`, or `Qexpr`.  As a simple example, `"+ 1 2"` is going to get stored as `Sexpr(Sym("+"), Num("1"), Num("2"))`.

This code makes use of the `Box` pointer type, which is a smart pointer to a heap-allocated value.  Because an `Lval` can hold many different types of data, the size of a given `Lval` is not known at compile-time.  By only storing pointers to values on the heap, we can build lists of them.  Because these `Box`es adhere to Rust's ownership and borrowing semantics, Rust is going to manage cleaning them up for us when they are no longer needed.  This is how we'll manage our memory over the lifetime of the program - with quite a bit less ceremony than the corresponding C!  If we *move* a `Box<Lval>` into a function during a function call and that function returns a *brand new* `Box<Lval>`, the old one will be detected as no longer in use and dropped automatically at the end of that function.  To build a new one, we use a constructor.  For example:

```rust
pub fn lval_num(n: i64) -> Box<Lval> {
    Box::new(Lval::Num(n))
}
```

Theres one of these for each variant.  Calling this will allocate the appropriate space with `Box::new()` on the heap and return the pointer.  No need to futz with a destructor - the `Box` will drop itself as soon as it can.

The containing types start out with an empty `Vec` of children, and can be manipulated with `lval_add` and `lval_pop`:

```rust
// Add lval x to lval::sexpr or lval::qexpr v
pub fn lval_add(v: &mut Lval, x: Box<Lval>) -> Result<()> {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            children.push(x);
        }
        _ => return Err(BlisprError::NoChildren),
    }
    Ok(())
}

// Extract single element of sexpr at index i
pub fn lval_pop(v: &mut Lval, i: usize) -> BlisprResult {
    match *v {
        Lval::Sexpr(ref mut children) | Lval::Qexpr(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            Ok(ret)
        }
        _ => Err(BlisprError::NoChildren),
    }
}
```

Both of these functions mutate their first argument in place, either removing or adding a child.  `Lval_add` consumes the second arg by taking ownership - it cannot be used again after adding it to another Lval because it is now owned by the containing `Lval`.

## Errors

One difference from the book's implementation is that I don't have a separate specific `Lval::Err` type for handling errors in our program.  Instead, I built a separate error type and leverage `Result<T, E>`-style error handling throughout:

```rust
#[derive(Debug)]
pub enum BlisprError {
    DivideByZero,
    EmptyList,
    LockError,
    NoChildren,
    NotANumber,
    NumArguments(usize, usize),
    ParseError(String),
    ReadlineError(String),
    WrongType(String, String),
    UnknownFunction(String),
}
```

To simplify the type signatures used throughout, I have a few type aliases:

```rust
pub type Result<T> = std::result::Result<T, BlisprError>;
pub type BlisprResult = Result<Box<Lval>>;
```

The majority of evaluation functions are going to return a `Result<Box<Lval>, BlisprError>`, now I can just type `BlisprResult`.  The few here and there that don't have a success type of `Box<LVal>` can still use this new `Result<T>` alias instead of the more verbose built-in `Result<T, E>`, and the error type will automatically always be this `BlisprError`.  WHich is what we want!  In order to be able to use this throughout our entire program, I've provided `impl From<E> for BlisprError` for a few other types of errors that are thrown, like `std::io::Error` and `pest::error::Error` for example:

```rust
impl<T> From<pest::error::Error<T>> for BlisprError
where
    T: Debug + Ord + Copy + Hash,
{
    fn from(error: pest::error::Error<T>) -> Self {
        BlisprError::ParseError(format!("{}", error))
    }
}

impl From<std::io::Error> for BlisprError {
    fn from(error: std::io::Error) -> Self {
        BlisprError::ParseError(error.to_string())
    }
}
```

This way I can still use the `?` operator on function calls that return these other error types inside functions that return a `BlisprResult`, and any errors returned will be automatically converted to the proper `BlisprError` for me.  Instead of storing specific error-type `Lval`s during our evaluation that are carried through the whole computation and finally printed out, all errors are bubbled up through the type system, but you still get the full `pest`-generated error carried along:

```lisp
blispr> {{(+}}
Parse error:  --> 1:5
  |
1 | {{(+}}
  |     ^---
  |
  = expected expr
blispr>
```

Full disclosure: to write the `pest::error::Error<T>` block, I just wrote what I wanted, i.e. `BlisprError::ParseError(format!("{}", error))` and appeased the compiler.  There is likely a better way to go about this but it works!

## Parsing

The book uses the author's own parser combinator library called [mpc](https://github.com/orangeduck/mpc).  If I were to tackle another similar problem in C, I'd likely reach for it again.  Rust, however, has its own strong ecosystem for parsing.  The two heavyweights in this space are [nom](https://github.com/Geal/nom) and [pest](https://github.com/pest-parser/pest).  For this project I opted for pest, to stay as close to the source material as possible.  Whereas `nom` will have you defining your own [parser combinators](https://dev.to/deciduously/parser-combinators-are-easy-4bjm), with `pest` you provide a PEG (or [Parsing Expression Grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar)), separately from your code.  Pest then uses Rust's powerful custom derive tooling to create a parse for your grammar automatically.

Here's the grammar I used for this language:

```pest
COMMENT = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
WHITESPACE = _{ " " }

num = @{ int }
    int = { ("+" | "-")? ~ digit+ }
    digit = { '0'..'9' }

symbol = @{ (letter | digit | "_" | arithmetic_ops | "\\" | comparison_ops | "&")+ }
    letter = { 'a' .. 'z' | 'A' .. 'Z' }
    arithmetic_ops = { "+" | "-" | "*" | "/" | "%" | "^" }
    comparison_ops = { "=" | "<" | ">" | "!" }

sexpr = { "(" ~ expr* ~ ")" }

qexpr = { "{" ~ expr* ~ "}" }

expr = { num | symbol | sexpr | qexpr }

blispr = { SOI ~ expr* ~ EOI }
```

This is stored in its own file called `blispr.pest` alongside the source code.  Each line refines a parse rule.  I find this exceedingly readable, and easy to tweak.  Starting from the bottom, we see a unit of valid `blispr` consists of one or more `expr`s between the Start of Input (SOI) and End of Input (EOI).  An `expr` is any of the options given.  It can handle comments and whitespace for you.  I also enjoy how the grammar maintained completely separately from any Rust code.  It's easy to get this working with Rust:

```rust
use pest::{iterators::Pair, Parser};

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blispr.pest");

#[derive(Parser)]
#[grammar = "blispr.pest"]
pub struct BlisprParser;
```

Now we can use the `BlisprParser` struct to parse string input into an AST.  In order to evaluate it, though, we need to build a a big `Lval`:

```rust
fn lval_read(parsed: Pair<Rule>) -> BlisprResult {
    match parsed.as_rule() {
        Rule::blispr | Rule::sexpr => {
            let mut ret = lval_sexpr();
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::qexpr => {
            let mut ret = lval_qexpr();
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::num => Ok(lval_num(parsed.as_str().parse::<i64>()?)),
        Rule::symbol => Ok(lval_sym(parsed.as_str())),
        _ => unreachable!(), // COMMENT/WHITESPACE etc
    }
}
```

At the bottom, `lval_eval` takes a string input.  We parse it into an AST using `BlisprParser::parse()`, which attempts to build the tree using the grammar we provided.  One AST will always correspond to one `Lval`.  We pass the parsed AST into `lval_read`, which will recursively build it for us.  This function looks at the top-level rule and takes an appropriate action.  The top-level rule, `blispr`, is treated as an S-expression, and for an S-expression we allocate a new `Lval` with `lval_sexpr()`. Then every child in the AST is added as a child to this containing `Lval`, passing through `lval_read()` itself to turn it into the correct `Lval`.  The rule for `qexpr` is similar, and the other rules just create the corresponding `Lval` from the type given.  The one weird one is `Rule::expr` - this is a sort of meta-rule that matches any of the valid expression types, so it's not its own lval, just wrapping one of a more specific type.  We just use `next()` to pass the actual rule found back into `lval_read()`.

The result of `lval_read()` will be a single `Lval::Sexpr` containing the entire parsed program, saved in `lval_ptr`.  Then we call `lval_eval()`, which will also return a `BlisprResult`.  If the evaluation is successful we just print out the result, and if any error was raised we print that error instead.

## Environment

Oh indextree, oh indextree...

Before we dig into how `lval_eval()` does its mojo lets pause and talk about the environment.  This is how symbols are able to correspond to functions and values - otherwise `"+"` would just be that character, but we need to to specifically correspond to the addition function.

Jury's out on whether or not I have the right idea, here, but I also handled this differently from the book.  The original text has you create a `struct` that holds two arrays and a counter, one for keys and the other for values.  To perform a lookup, you find the index of that key and then return the value at that same index in the values.  This struct is built before the program enters the loop, and is passed in manually to every single function that gets called.

Instead, I decided to leverage the [`lazy_static`](https://github.com/rust-lang-nursery/lazy-static.rs) crate.  This allows you to define `static` values that require runtime initialization like heap allocation.

I've also opted for a [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) data structure instead of two separated arrays with matching indices:

```rust
lazy_static! {
    pub static ref ENV: LenvStore<'static> = Arc::new(RwLock::new(Lenv::new(None)));
}

pub type LenvStore = Arc<RwLock<Lenv>>;

#[derive(Debug, Clone)]
pub struct Lenv {
    lookup: HashMap<String, Box<Lval>>,
    parent: Option<LenvStore>,
}
```

The `Lenv` itself only holds the lookup table, but when we initialize it in the `lazy_static!` block it gets wrapped in an `Arc<RwLock<T>>`.  The `Arc` is a reference-counted type - it will get de-allocated when no further references to it exist.  Cloning an `Arc` just returns a new pointer to the same data on the heap and increments this counter, and when the new clone goes out of scope the counter will auto-decrement.  Great.  The `RwLock` is a special type of mutex that allows either multiple concurrent readers or a single writer at a time, but not both.  This makes it ideal for use as the `parent` pointer type.  To use it, we can just make a new clone without worrying taht we're needlessly copying the data, and the `Arc` will neatly destruct itself when appropriate.  At present this code doesn't take advantage of any parallel execution, but having the environment thus typed should make that process easier as well.

I've got some helper methods for getting, setting, and enumerating the contents:

```rust
impl Lenv {
    // ..

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
```

I did have to manually implement `PartialEq` for these ones and unwrap the parents myself:

```rust
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
```

Getting a value from the environment will return a brand new `Lval` with a copy of what's stored, and printing out the contents will also return a ready-made `Qexpr` lval containing `Symbol`s corresponding to each entry.  We'll come back to initialization after talking a bit about evaluation.

Environments optionally hold a parent environment, and if the lookup fails in this one it will attempt the parent environment.

## Eval

The `lval_eval()` called in `eval_str()` is where the real crunching happens.  This will take an `Lval` (that is, an AST) and recursively evaluate it to a final `Lval`.  Most types of `Lval` are already evaluated fully - but any `S-Expression` found will need to be evaluated, and any `Symbol` gets looked up in the environment.

Before looking at the Rust, let's break it down in English:

1. Check the type of Lval:

    a. Fun | Num | Qexpr - we're done - return lval as is.

    b. Symbol - Do an environment lookup with `Lenv::get()` - e.g., for `Sym("+")`, see if we have a function pointer stored at name `"+"`.  Return result of lookup, which will already be an `Lval`.

    c. Sexpr - Evaluate the S-Expression.

2. If we made it to this step, we're working with an S-Expression.  Everything else has already returned. Before going further, fully evaluate all children with `lval_eval()`.

3. 

Here's the whole thing:

```rust
pub fn lval_eval(mut v: Box<Lval>) -> BlisprResult {
    let child_count;
    match *v {
        Lval::Sym(s) => {
            let r = ENV.read()?;
            let result = r.get(&s)?;
            debug!(
                "lval_eval: Symbol lookup - retrieved {:?} from key {}",
                result, s
            );
            return Ok(result);
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
                LvalFun::Builtin(f) => f(v),
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
```
