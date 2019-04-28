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
   Compiling blispr v0.0.1 (/home/yourstruly/code/blispr)
    Finished dev [unoptimized + debuginfo] target(s) in 0.92s
     Running `target/debug/blispr`
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
blispr> (add (* 8 7/* comments too! */ (div 65 5) (- 21 3 11)) (max 8 2 (min 3 76)))
5104
blispr>
```

It uses [`rustyline`](https://github.com/kkawakam/rustyline) as a readline alternative which will save history to `./.blispr-history.txt`.  See that repo for all supported options.

You can pass `-d` at runtime (`cargo run -- -d` or `blispr -d`) to enable overly verbose debug output:
```
$ cargo run -- -d
   Compiling blispr v0.0.1 (/home/yourstruly/code/blispr)
    Finished dev [unoptimized + debuginfo] target(s) in 1.21s
     Running `target/debug/blispr -d`
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
 DEBUG blispr::parse > Debug mode enabled
blispr> eval (list + 3 4 (* 6 2)/* or whatever */ )
 DEBUG blispr::parse > blispr(0, 44, [expr(0, 4, [symbol(0, 4)]), expr(5, 43, [sexpr(5, 43, [expr(6, 10, [symbol(6, 10)]), expr(11, 12, [symbol(11, 12)]), expr(13, 14, [num(13, 14)]), expr(15, 16, [num(15, 16)]), expr(17, 24, [sexpr(17, 24, [expr(18, 19, [symbol(18, 19)]), expr(20, 21, [num(20, 21)]), expr(22, 23, [num(22, 23)])])])])]), EOI(44, 44)])
 DEBUG blispr::parse > Parsed: (eval (list + 3 4 (* 6 2)))
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("eval"), Sexpr([Sym("list"), Sym("+"), Num(3), Num(4), Sexpr([Sym("*"), Num(6), Num(2)])])])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("eval")
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("list"), Sym("+"), Num(3), Num(4), Sexpr([Sym("*"), Num(6), Num(2)])])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("list")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("+")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(3)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(4)
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("*"), Num(6), Num(2)])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("*")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(6)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > Calling function * on Sexpr([Num(6), Num(2)])
 DEBUG blispr::eval  > Multiply 6 and 2
 DEBUG blispr::eval  > Calling function list on Sexpr([Sym("+"), Num(3), Num(4), Num(12)])
 DEBUG blispr::eval  > Building list from [Sym("+"), Num(3), Num(4), Num(12)]
 DEBUG blispr::eval  > Calling function eval on Sexpr([Qexpr([Sym("+"), Num(3), Num(4), Num(12)])])
 DEBUG blispr::eval  > builtin_eval: Sexpr([Sym("+"), Num(3), Num(4), Num(12)])
 DEBUG blispr::eval  > lval_eval: Sexpr([Sym("+"), Num(3), Num(4), Num(12)])
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Sym("+")
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(3)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(4)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(12)
 DEBUG blispr::eval  > Calling function + on Sexpr([Num(3), Num(4), Num(12)])
 DEBUG blispr::eval  > Add 3 and 4
 DEBUG blispr::eval  > Add 7 and 12
 DEBUG blispr::parse > Result: Num(19)
19
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
blispr> cons 3 {4 5}
{3 4 5}
blispr> init {1 2 3 4}
{1 2 3}
```

...that's it!

Only accepts integers for now, decimal points in numbers are a syntax error.