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
blispr> (add (* 8 7/* comments too! */ (div 65 5) (- 21 3 11)) (max 8 2 (min 3 76)))
5104
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

You can enable overly verbose debug logging with `$ RUST_LOG=blispr=debug cargo run`:

```
blispr> eval (list + 1 2)
 DEBUG blispr::parse > blispr(0, 17, [expr(0, 4, [symbol(0, 4)]), expr(5, 17, [sexpr(5, 17, [expr(6, 10, [symbol(6, 10)]), expr(11, 12, [symbol(11, 12)]), expr(13, 14, [num(13, 14)]), expr(15, 16, [num(15, 16)])])]), EOI(17, 17)])
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("eval"), Sexpr([Sym("list"), Sym("+"), Num(1), Num(2)])])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("eval")
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("list"), Sym("+"), Num(1), Num(2)])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("list")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("+")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(1)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > Calling function list on Sexpr([Sym("+"), Num(1), Num(2)])
 DEBUG blispr::eval  > Building list from [Sym("+"), Num(1), Num(2)]
 DEBUG blispr::eval  > Calling function eval on Sexpr([Qexpr([Sym("+"), Num(1), Num(2)])])
 DEBUG blispr::eval  > builtin_eval: Sexpr([Sym("+"), Num(1), Num(2)])
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("+"), Num(1), Num(2)])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("+")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(1)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > Calling function + on Sexpr([Num(1), Num(2)])
 DEBUG blispr::eval  > Add 1 and 2
 DEBUG blispr::parse > Result: Num(3)
3
blispr> 
```

## Currently implemented

Operators: `+ | add`, `- | sub`, `* | mul`, `/ | div`, `% | rem`, `^ | pow`, `max`, `min`

```
blispr> list 1 2 3
{1 2 3}
blispr> eval {+ 1 2}
3
blispr> join {1 2} {3 4}
{1 2 3 4}
blispr> len {1 2 3 4 5}
5
blispr> head {1 2 3}
1
blispr> tail {1 2 3}
{2 3}
```

...that's it!

Only accepts integers for now, decimal points in numbers are a syntax error.