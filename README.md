# blispr

[![Build Status](https://travis-ci.org/deciduously/blispr.svg?branch=master)](https://travis-ci.org/deciduously/blispr)

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
Use exit(), Ctrl-C, or Ctrl-D to exit prompt
blispr> (def {x} 100)
()
blispr> (def {y} 200)
()
blispr> (def {a /* comments look like this */ b} 5 6)
()
blispr> (eval (cons (head {+ - * /}) (list a b x y)))
311
```

Newlines are whitespace as well.  The surrounding parens are required - a valid blispr program is one or more forms.  Omitting them will result in only the final form getting returned:

```
blispr> + 4 5
5
blispr> (+ 4 5)
9
```

The first attempt was evaluated as follows:

```
blispr> +
<builtin: +>
blispr> 4
4
blispr> 5
5
```

It uses [`rustyline`](https://github.com/kkawakam/rustyline) as a readline alternative which will save history to `./.blispr-history.txt`.  See that repo for all supported options.

Run with no arguments for the repl, or pass an input file with `-i` or `--input`:

test.blispr:

```
(def {x} 100)
(def {y} 200)
(def {a b} 5 6)
(eval (cons (head {+ - * /}) (list a b x y)))
```

```
$ cargo run -- -i test.blispr 
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/blispr -i test.blispr`
311
```

You can pass `-d` or `--debug` at runtime (`cargo run -- -d` or `blispr -d`) to enable overly verbose debug output:

```
$ cargo run -- -d
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/blispr -d`
Blispr v0.0.1
Use exit(), Ctrl-C, or Ctrl-D to exit prompt
 DEBUG blispr::parse > Debug mode enabled
blispr> (eval (list + 1 2))
 DEBUG blispr::parse > blispr(0, 19, [expr(0, 19, [sexpr(0, 19, [expr(1, 5, [symbol(1, 5)]), expr(6, 18, [sexpr(6, 18, [expr(7, 11, [symbol(7, 11)]), expr(12, 13, [symbol(12, 13)]), expr(14, 15, [num(14, 15)]), expr(16, 17, [num(16, 17)])])])])]), EOI(19, 19)])
 DEBUG blispr::parse > Parsed: Blispr([Sexpr([Sym("eval"), Sexpr([Sym("list"), Sym("+"), Num(1), Num(2)])])])
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Symbol lookup - retrieved Fun(Builtin(eval)) from key "eval"
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Symbol lookup - retrieved Fun(Builtin(list)) from key "list"
 DEBUG blispr::eval  > lval_eval: Symbol lookup - retrieved Fun(Builtin(+)) from key "+"
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(1)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > Calling function Fun(Builtin(list)) on Sexpr([Sym("list"), Sym("+"), Num(1), Num(2)])
 DEBUG blispr::eval  > builtin_list: Building qexpr from [Fun(Builtin(+)), Num(1), Num(2)]
 DEBUG blispr::eval  > Calling function Fun(Builtin(eval)) on Sexpr([Sym("eval"), Sexpr([Sym("list"), Sym("+"), Num(1), Num(2)])])
 DEBUG blispr::eval  > builtin_eval: Sexpr([Fun(Builtin(+)), Num(1), Num(2)])
 DEBUG blispr::eval  > lval_eval: Sexpr, evaluating children
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Fun(Builtin(+))
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(1)
 DEBUG blispr::eval  > lval_eval: Non-sexpr: Num(2)
 DEBUG blispr::eval  > Calling function Fun(Builtin(+)) on Sexpr([Fun(Builtin(+)), Num(1), Num(2)])
 DEBUG blispr::eval  > builtin_op: Add 1 and 2
3
```

## Currently implemented

* Operators: `+ | add`, `- | sub`, `* | mul`, `/ | div`, `% | rem`, `^ | pow`, `max`, `min`.  Aliases point to the same function.

* Utilties: `printenv(), exit()` (must be passed with an argument - an empty S-Expression works):

```
blispr> _def {a b c d e f g h i j k l m n o p q r s t u v w x y z} 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26)
()
blispr> (def {func} (\ {a b} {+ a b}))
()
blispr> (printenv())
{g:7 m:13 o:15 head:<builtin: head> *:<builtin: *> f:6 list:<builtin: list> p:16 printenv:<builtin: printenv> d:4 tail:<builtin: tail> ^:<builtin: ^> cons:<builtin: cons> j:10 sub:<builtin: sub> q:17 init:<builtin: init> s:19 +:<builtin: +> %:<builtin: %> t:20 /:<builtin: /> v:22 w:23 y:25 z:26 func:(\ {a b} {+ a b}) mul:<builtin: mul> join:<builtin: join> exit:<builtin: exit> rem:<builtin: rem> add:<builtin: add> def:<builtin: def> pow:<builtin: pow> h:8 div:<builtin: div> \:<builtin: \> max:<builtin: max> b:2 l:12 n:14 r:18 x:24 k:11 e:5 u:21 eval:<builtin: eval> -:<builtin: -> min:<builtin: min> c:3 i:9 len:<builtin: len> a:1}
blispr> exit()
Goodbye!

$
```

* List operations:

```
blispr> (list 1 2 3)
{1 2 3}
blispr> (eval {+ 1 2})
3
blispr> (join {1 2} {3 4})
{1 2 3 4}
blispr> (len {1 2 3 4 5})
5
blispr> (head {1 2 3})
1
blispr> (tail {1 2 3})
{2 3}
blispr> (cons 3 {4 5})
{3 4 5}
blispr> (init {1 2 3 4})
{1 2 3}
```

* Variable defintions - new assignments to the same binding will overwrite old ones, there's just one big global scope:

```
blispr> (def {x} 100)
()
blispr> (def {y} 200)
()
blispr> (def {a b} 5 6)
()
blispr> (def {arglist} {a b x y})
()
blispr> arglist
{a b x y}
blispr> (+ a b x y)
311
blispr> (def arglist 1 2 3 4)
()
blispr> (list a b x y)
{1 2 3 4}
```

* User-defined lambdas

Now with partial application!

```
blispr> (def {embiggen} (\ {x y} {^ (* x y) (+ x y)}))
()
blispr> (embiggen 2 3)
7776
blispr> (def {real-big} (embiggen 4))
()
blispr> (real-big 2)
262144
blispr> 
```

...that's it!

Only accepts integers for now, decimal points in numbers are a syntax error.
