# Slog

This is a reimplementation of [slog](https://github.com/harp-lab/slog-lang1). This project compiles a subset of slog to another datalog dialect called [ascent](https://github.com/s-arash/ascent) via procedural macros in rust. Since its a pure macro implementation, user can get the same "seamless" experience of switching between slog, ascent and even rust. To read this documentation, you should be familiar with the syntax of slog and ascent.

## Syntax

### Signature
```rust
slog! {
    (struct Foobar)
    // ...
}
```
You can declare a slog program by wrapping your slog program in a `slog!` macro. By `struct` keyword, you can associate a name with your program. Your program will be expanded to a rust struct with the same name. The generated rust stuct will have exact the same behavior as `ascent`, for more detail, please refer to documentation of `ascent`.
Here is the expanded code in ascent:
```rust
ascent! {
    struct Foobar;
}
```

### Define
```rust
slog! {
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)
}
```
In this implementation of slog, you have to declare all the relations and type of their arities before you can use them. Slightly different from ascent and other datalog dialects, each relation will be associated with a hidden relation named `<relation_name>_id`. This id relation will have an extra ID column serve as the primary key of the relation. Under the hood, above code will be expanded to following code in ascent:
```rust
ascent! {
    relation foo (usize , usize) ;
    relation foo_id (usize , usize, usize) ;
    //  we modify ascent to support ID syntax sugar.
    //  above code can be simplified to:
    relation ID foo (usize , usize) ;
    // ...
}
```
To support structural facts in slog, we provide a syntax sugar `sexpr` for structural facts type in relation, it will be desugared into `usize` in ascent.

### ID
Different from ascent, slog's semantic builts on $DL^{\exists!}$, for more detail semantic, please refer to [slog paper](https://arxiv.org/pdf/2411.14330v2). To support this semantic, we modify the ascent to support via `let` keyword in head of rules.
```rust
ascent! {
    let x = foo (a , b) <-- bar(a, b); // same as ∃! x. foo(a, b) ← ∃!y. bar(a, b)
}
```
As we talked above, the ID support is under the hood implemented by an extra shadow id relation for each relation.


### Facts

To insert a fact, you can use `#` prefix to an sexpr like below:

```rust
slog! {
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)
    #(foo 1 2)
    #(bar 3 4)
    #(foobar (foo 1 2) (bar 3 4))
    #(foobar ?(foo x y) ?(bar a b))
}
```

Both normal fact and nested facts in slog can be supported. Moreover, we also support the slog's `?` operator in facts, which will do unfication in the fact rule.

```rust
// expanded code in ascent:
ascent! { pub struct Foobar ;
    relation ID foo (usize , usize) ;
    relation ID bar (usize , usize) ;
    relation ID foobar (usize , usize) ;
    let foo_784959 = foo (1 , 2) <-- () ;
    let bar_440915 = bar (3 , 4) <-- () ;
    let foobar_608073 = foobar (foo_935778 , bar_489815) <- - foo (x , y) . foo_935778 , bar (a , b) . bar_489815 ;
}
```

### Vanilla Datalog Rule

The simplest datalog rule syntax can be supported in slog's sexpr flavor:
```rust
slog! {
    (struct TC)
    (define edge usize usize)
    (define tc usize usize)

    [(tc x y) <-- (edge x y)]
    // [(edge x y) --> (tc x y)]
    [(tc x y) <-- (edge x y) (tc x y)]
}
```
The reverse arrow syntax in original implementation of slog is also supported in this implementation, which will reverse the order of the body and heads in the rule.

```rust
// expanded code in ascent:
ascent! { pub struct TC ;
relation ID edge (usize , usize) ;
relation ID tc (usize , usize) ;
let tc_333141 = tc (x , y) <-- edge (x , y) ;
let tc_639465 = tc (x , y) <-- edge (x , y) , tc (x , y) ;
}
```

### Seamless ascent/rust/slog coding experience

To allow switching between slog/ascent so you can get expose to all powerful rust/ascent syntax, we allow escape syntax via quasiquote in slog.

```rust
slog! {
    (struct TC)
    (define edge usize usize)
    (define tc usize usize)

    [(tc x y) <-- (edge x y) ,(let y = y + 1)]
}
```
Inside quaiquote, you can write any ascent code, it will be directly expanded to ascent code.
```rust
// expanded code in ascent:
ascent! { pub struct TC ;
relation ID edge (usize , usize) ;
relation ID tc (usize , usize) ;
let tc_333141 = tc (x , y) <-- edge (x , y) , let y = y + 1 ;
}
```
This syntax works both in clause and relation argument position.
Escaping syntax make us get rid of the need of implementing primitives like `+-*/` and `<,>=,==,...` in slog.
```rust
slog! {
    // ...
    [(tc x y) <-- (edge x y) ,(if y > &10)]
    [(tc ,(x+1) y) <-- (edge x y)]
}
```

### Unification of ID

We also support explicit use and rename the hidden id of a relation by `=` operator over a body clause, which is useful for ID unification.

```rust
slog! {
    (struct Foobar)
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    [(foobar idf (bar x y)) <-- (= idf (foo x y)) (bar x y)]
}
```

```rust
// expanded code in ascent:
ascent! { pub struct Foobar ;
relation ID foo (usize , usize) ;
relation ID bar (usize , usize) ;
relation ID foobar (usize , usize) ;
let foobar_608073 = foobar (idf , bar_489815) <- - foo (x , y) . idf , bar (x , y) ;
}
```
`foo (x , y) . idf` is another syntax sugar we added in ascent for $DL^{\exists!}$ semantic.

### Structured Clause

The most powerful feature of slog is you can use structral fact to give you algebraic data type like experience.
```rust
slog! {
    (struct Foobar)
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    [(foobar (foo x y) (bar a b)) <-- (foo x y) (bar a b) (foobar (foo x y) (bar a b))]
}
```

In this implementation, we will compiling as we talked in slog paper via a linked list like fact chaining.

```rust
// expanded code in ascent:
ascent! { pub struct Foobar ;
relation ID foo (usize , usize) ;
relation ID bar (usize , usize) ;
relation ID foobar (usize , usize) ;
let foo_665590 = foo (x , y) , let bar_465066 = bar (a , b) , let foobar_355658 = foobar (foo_665590 , bar_465066)
    <-- 
    foo (x , y) , bar (a , b) ,
    foobar (foo_742117 , bar_751350) ,
    foo (x , y) . foo_742117 ,
    bar (a , b) . bar_751350 ;
}

```

## Slog Syntax Sugar

We support "!/?" syntax sugar as same as original slog.

## Egglog-like Equivalence (On the way)

We support egglog's EGraph and inflation operation in slog.
