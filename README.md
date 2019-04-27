# blispr

It's [blisp](https://github.com/deciduously/blisp), but in Rust!   Blispr.

This is a rewrite of the Lisp built in the book [Build Your Own Lisp](http://www.buildyourownlisp.com/) in Rust.  The end product is superficially similar, but there are some places where I have either by choice or necessity implemented something differently - different enough that I can't accurately call this a port of that tutorial.  It's just the world's latest unnecessary-est lisp!

## Requirements

* Rust (stable)

## Usage

```blispr
$ git clone https://github.com/deciduously/blispr
$ cd blispr
$ cargo run
   Compiling blispr v0.0.1 (file:///home/yourstruly/code/blispr)
    Finished dev [unoptimized + debuginfo] target(s) in 0.92s
     Running `target/debug/blispr`
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
blispr> (add (* 8 7/* comments too! */ (div 7 8) (- 2 3 1)) (max 8 2 (min 8 7)))
2
blispr>
```

It uses `rustyline` as a readline alternative which will save history to `./.blisp-history.txt`.

You can pass `-p` at runtime (`cargo run -- -p` or `blispr -p`) to display the parsed input, pre-eval alongside the result:

```blispr
$ cargo run -- -p
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
Debug mode enabled
blispr> + /* apes?! */ 1    2
(+ 1 2)
3
blispr>
```

## Currently implemented

* Operators: `+ | add`, `- | sub`, `* | mul`, `/ | div`, `% | rem`, `^ | pow`, `max`, `min`

* `list`:

```
blispr> list 1 2 3
{1 2 3}
```

...that's it!  `eval` and `join` are coming, it's gonna be off the chain.

Only accepts integers for now, decimal points in numbers are a syntax error.