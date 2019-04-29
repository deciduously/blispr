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
blispr> def {x} 100
()
blispr> def {y} 200
()
blispr> def {a /* comments look like this */ b} 5 6
()
blispr> eval (cons (head {+ - * /}) (list a b x y))
311
```

It uses [`rustyline`](https://github.com/kkawakam/rustyline) as a readline alternative which will save history to `./.blispr-history.txt`.  See that repo for all supported options.

Run with no arguments for the repl, or pass an input file with `-i` or `--input`:

test.blispr:

```
def {x} 100
def {y} 200
def {a b} 5 6
eval (cons (head {+ - * /}) (list a b x y))
```

```
$ cargo run -- -i test.blispr
   Compiling blispr v0.0.1 (/home/ben/code/blispr)
    Finished dev [unoptimized + debuginfo] target(s) in 6.83s
     Running `target/debug/blispr -i test.blispr`
()
()
()
311
```

You can pass `-d` or `--debug` at runtime (`cargo run -- -d` or `blispr -d`) to enable overly verbose debug output:

```
± |master U:8 ?:1 ✗| → cargo run -- -d
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/blispr -d`
Blispr v0.0.1
Press Ctrl-C or Ctrl-D to exit
 DEBUG blispr::parse > Debug mode enabled
blispr> (eval (head {^ + - + - * / /* its homoiconic! */})) 2 32
 DEBUG blispr::parse > blispr(0, 56, [expr(0, 51, [sexpr(0, 51, [expr(1, 5, [symbol(1, 5)]), expr(6, 50, [sexpr(6, 50, [expr(7, 11, [symbol(7, 11)]), expr(12, 49, [qexpr(12, 49, [expr(13, 14, [symbol(13, 14)]), expr(15, 16, [symbol(15, 16)]), expr(17, 18, [symbol(17, 18)]), expr(19, 20, [symbol(19, 20)]), expr(21, 22, [symbol(21, 22)]), expr(23, 24, [symbol(23, 24)]), expr(25, 26, [symbol(25, 26)])])])])])])]), expr(52, 53, [num(52, 53)]), expr(54, 56, [num(54, 56)]), EOI(56, 56)])
 DEBUG blispr::parse > Parsed: Sexpr([Sexpr([Sym("eval"), Sexpr([Sym("head"), Qexpr([Sym("^"), Sym("+"), Sym("-"), Sym("+"), Sym("-"), Sym("*"), Sym("/")])])]), Num(2), Num(32)])
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Qexpr([Sym("^"), Sym("+"), Sym("-"), Sym("+"), Sym("-"), Sym("*"), Sym("/")])
 DEBUG blispr::eval  > Calling function Fun(0x55da8de1d040) on Sexpr([Qexpr([Sym("^"), Sym("+"), Sym("-"), Sym("+"), Sym("-"), Sym("*"), Sym("/")])])
 DEBUG blispr::eval  > builtin_head: Returning the first element of [Sym("^"), Sym("+"), Sym("-"), Sym("+"), Sym("-"), Sym("*"), Sym("/")]
 DEBUG blispr::eval  > Calling function Fun(0x55da8de1c4f0) on Sexpr([Sym("^")])
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(32)
 DEBUG blispr::eval  > Calling function Fun(0x55da8de19d60) on Sexpr([Num(2), Num(32)])
 DEBUG blispr::eval  > builtin_op: Raise Num(2) to the Num(32) power
4294967296
```

## Currently implemented

* Operators: `+ | add`, `- | sub`, `* | mul`, `/ | div`, `% | rem`, `^ | pow`, `max`, `min`

* List operations:

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

* Variable defintions - new assignments to the same binding will overwrite old ones, there's just one big global scope:

```
blispr> def {x} 100
()
blispr> def {y} 200
()
blispr> def {a b} 5 6
()
blispr> def {arglist} {a b x y}
()
blispr> arglist
{a b x y}
blispr> + a b x y
311
blispr> def arglist 1 2 3 4
()
blispr> list a b x y
{1 2 3 4}
```

...that's it!

Only accepts integers for now, decimal points in numbers are a syntax error.