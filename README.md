# blispr

It's [blisp](https://github.com/deciduously/blisp), but in Rust!   Blispr.

This is a rewrite of the Lisp built in the book [Build Your Own Lisp](http://www.buildyourownlisp.com/) in Rust.  The end product is superficially similar, but there are some places where I have either by choice or necessity implemented something differently.

The most significant differnence at this stage is my use of the more Rust-friendly [Pest](https://pest.rs) instead of the book author's own [mpc](https://github.com/orangeduck/mpc).  I may try to write an alternate branch using that parser to compare.  The resulting code is pretty similar but `mpc` specifies the grammar inside the source file whereas `pest` keeps it in a separate `src/blispr.pest`.

Most other differneces stem from this library choice as well, making this library different enough that I can't accurately call this a port of that tutorial.  It's just the world's latest unnecessy-est lisp!

## Requirements

* Rust (stable)

## Usage

```shell
git clone https://github.com/deciduously/blispr
cd blispr
cargo run
   Compiling blispr v0.0.1 (file:///home/cooldude/code/blispr)
    Finished dev [unoptimized + debuginfo] target(s) in 0.92s
     Running `target/debug/blispr`
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
blispr> (add (* 8 7/* comments too! */ (div 7 8) (- 2 3 1)) (max 8 2 (min 8 7)))
2
blispr>
```



It uses `rustyline` as a readline alternative which will save history to `./.blisp-history.txt`.

## Currently implemented

* Operators: `+` | `add`, `-` | `sub`, `*` | `mul`, `/` | `div`, `max`, `min`

...that's it!