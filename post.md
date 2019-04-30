# Rust Your Own Lisp

I whet my appetite for translating things by [orangeduck](http://theorangeduck.com/page/about) into [Rust](https://www.rust-lang.org/) with the [Markov Chain](https://dev.to/deciduously/build-you-a-markov-chain-in-rust-or-whatever-54mo), but that project really just scrathed the itch.  His book [Build Your Own Lisp](http://www.buildyourownlisp.com/) is fantastic, both as an introduction to C and an introduction to writing an interpreter, so naturally I had to give it a stab.

This post is not intended to be a replacement - go read the book, its excellent.  In translating to Rust, though, there are a few differences in the code bases that are worth noting.

## Parsing

The book uses the author's own parser combinator library called [mpc](https://github.com/orangeduck/mpc).  If I were to tackle another similar problem in C, I'd likely reach for it again.  Rust, however, has its own strong ecosystem for parsing.  The two heavyweights in theis space are [nom](https://github.com/Geal/nom) and [pest](https://github.com/pest-parser/pest).  For this project I opted for pest, to stay as close to the source material as possible.  Whereas `nom` will have you defining your own [parser combinators](https://dev.to/deciduously/parser-combinators-are-easy-4bjm), with `pest` you provide a PEG (or Parsing Expression Grammar), separately from your code.  Pest then uses Rust's powerful custom derive tooling to create a parse for your grammar automatically.

Here's the grammar I used for this langauge:

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

This is stored in its own file called `blispr.pest` alongisde the source code.  Each line refines a parse rule.  I find this exceedingly readable, and easy to tweak.  It can handle comments and whitespace for you.  I also enjoy how it's maintained separately from any Rust code.  It's easy to get this working with your code:

```rust
use pest::{iterators::Pair, Parser};

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blispr.pest");

#[derive(Parser)]
#[grammar = "blispr.pest"]
pub struct BlisprParser;
```
