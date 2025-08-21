# Slog on $\texttt{Ascent}^{\exists!}$

This project is a reimplementation of the [slog](https://github.com/harp-lab/slog-lang1) programming language. It compiles a subset of Slog to another Datalog dialect, [Ascent](https://github.com/s-arash/ascent), using Rust procedural macros. Because it's a pure macro implementation, you get a seamless experience switching between Slog, Ascent, and even native Rust code.

This documentation assumes you are familiar with the basic syntax of both Slog and Ascent.

***

## Core Concepts: IDs and $DL^{\exists!}$

Slog's semantics are built on $DL^{\exists!}$, a logic that requires every derived fact to be unique. To support this, our implementation gives every fact a unique ID. Under the hood, each relation you define, like `foo`, is paired with a hidden shadow relation, `foo_id`, that includes an extra column for this unique ID.

We've modified Ascent to support this semantic with a `let` keyword in the head of a rule, which corresponds to the existential quantifier $\exists!$ ("there exists a unique...").

```rust
// This Ascent code:
let x = foo(a, b) <-- bar(a, b);

// Is equivalent to this logic:
// ∃! x. foo(a, b) ← ∃ y. bar(a, b)
```

This core concept of unique fact IDs enables Slog's most powerful features, including structured clauses and ID unification.

***

## Syntax

### Program and Relation Definition

You declare a Slog program by wrapping it in the `slog!` macro and giving it a name with the `struct` keyword. This expands into a Rust struct that behaves just like a standard Ascent program.

You must declare all relations and the types of their fields using the `define` keyword before using them. We provide a special `sexpr` type that acts as a syntax sugar for structural facts; it is desugared to `usize` in the compiled Ascent code.

**Slog:**
```rust
slog! {
    (struct MyProgram)
    (define edge usize usize)
    (define path sexpr sexpr) // 'sexpr' for structural facts
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    struct MyProgram;
    // Each relation gets a hidden ID field.
    // This can be expressed with our modified ID syntax sugar:
    relation ID edge(usize, usize);
    relation ID path(usize, usize);
}
```

---

### Asserting Facts

To insert a fact, prefix an s-expression with `#`. This works for both simple and nested structural facts. You can also use the `?` operator to perform unification directly within a fact definition.

**Slog:**
```rust
slog! {
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    #(foo 1 2)
    #(bar 3 4)

    // A nested structural fact
    #(foobar (foo 1 2) (bar 3 4))

    // A fact with unification
    #(foobar ?(foo x y) ?(bar a b))
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    /* ... relations ... */
    // Facts become rules with an empty body
    let foo_784959 = foo(1, 2) <-- ();
    let bar_440915 = bar(3, 4) <-- ();

    // Nested facts are unified with existing relations
    let foobar_608073 = foobar(foo_935778, bar_489815) <--
        foo(x, y).foo_935778,
        bar(a, b).bar_489815;
}
```
*Note: The `.` syntax (`foo(x, y).foo_935778`) is another sugar we added to Ascent to bind a fact's ID to a variable.*

---

### Writing Rules

#### Vanilla Datalog Rules

Standard Datalog rules are expressed using s-expressions. Both forward `(<--)` and reverse `(-->)` arrows are supported.

**Slog:**
```rust
slog! {
    (struct TC)
    (define edge usize usize)
    (define tc usize usize)

    // Standard rule
    [(tc x y) <-- (edge x y)]

    // Rule with a reverse arrow (same meaning)
    // [(edge x y) --> (tc x y)]

    // Rule with multiple body clauses
    [(tc x z) <-- (edge x y) (tc y z)]
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    pub struct TC;
    relation ID edge(usize, usize);
    relation ID tc(usize, usize);
    
    let tc_333141 = tc(x, y) <-- edge(x, y);
    let tc_983452 = tc(x, z) <-- edge(x, y), tc(y, z);
}
```

#### Structured Clauses

The most powerful feature of Slog is the ability to use relations within the head and body of rules, creating **structured clauses**. This gives you an experience similar to using algebraic data types. These rules are compiled into a chain of `let` bindings and ID unifications.

**Slog:**
```rust
slog! {
    (struct Foobar)
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    // This rule creates a new `foobar` fact from existing `foo` and `bar` facts.
    [(foobar (foo x y) (bar a b)) <--
        (foo x y)
        (bar a b)]
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    /* ... relations ... */
    // The structured head is compiled into multiple `let` bindings.
    // The body clauses are used for unification.
    let foo_665590 = foo(x, y),
    let bar_465066 = bar(a, b),
    let foobar_355658 = foobar(foo_665590, bar_465066)
    <--
        foo(x, y),
        bar(a, b);
}
```

#### Unification of IDs

You can explicitly capture and rename the hidden ID of a fact using the `=` operator on a body clause. This is useful for complex dataflow and unification scenarios.

**Slog:**
```rust
slog! {
    (struct Foobar)
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    // Capture the ID of `(foo x y)` as `idf` and use it in the head.
    [(foobar idf (bar x y)) <-- (= idf (foo x y)) (bar x y)]
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    /* ... relations ... */
    let foobar_608073 = foobar(idf, bar_489815) <--
        foo(x, y).idf,
        bar(x, y).bar_489815;
}
```

---

### Seamless Ascent/Rust Integration

To access the full power of Ascent and native Rust, you can escape the Slog syntax using quasiquotes: `,(...)`. Any code inside a quasiquote is directly embedded into the generated Ascent program. This is useful for performing arithmetic, comparisons, or calling Rust functions.

This feature eliminates the need to reimplement basic primitives like `+`, `-`, `*`, `/`, `<`, `>`, and `==` in Slog.

**Slog:**
```rust
slog! {
    (struct MyProgram)
    (define edge usize usize)
    (define path usize usize)

    // Use an escape to filter a rule
    [(path x y) <-- (edge x y) ,(if y > &10)]

    // Use an escape to perform arithmetic in the head
    [(path ,(x + 1) y) <-- (edge x y)]
}
```

**Expanded Ascent Code:**
```rust
ascent! {
    pub struct MyProgram;
    relation ID edge(usize, usize);
    relation ID path(usize, usize);

    let path_333141 = path(x, y) <-- edge(x, y), if y > &10;
    let path_639465 = path(x + 1, y) <-- edge(x, y);
}
```

***

## Additional Features

### Slog Syntax Sugar

#### `?` Operator
The `?` operator can let you query in the head of a rule. A head clause is true only if its question marked argument corresponds to a predicate with the same name holds in current context.
```rust
slog! {
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    [(foo ?(bar x z) z) <-- (foo x y)]
}
```
Above query is equivalent to:
```rust
slog! {
    (define foo usize usize)
    (define bar usize usize)
    (define foobar sexpr sexpr)

    [(foo bid z) <-- (foo x y) (= bid (bar x z))]
}
```

#### Join order control with `?` Operator in Body

Slightly different from original Slog, we also allow usage of `?` Operator in the body of a rule, but for join order control.
Join order of nested facts is completely random in original implementation of Slog, which can sometime cause program too hard to debug.
For example in following program:

```rust
slog! {
    // ...
    [(bar x y) <-- (foobar (foo x y) _) (bar x y)]
}
```
join order between `foo`, `bar` and `foobar` is completely random cause computation time also complete random. Even if you know foobar is small relation, you want join
foobar and foo first can save time, you have no way to control this unless you explicity unify the id of relation, which will
cause program hard to read.
```
// in slog 1.0
[(bar x y) <-- (foobar id_f _) -- (= idf (foo x y)) -- (bar x y)]
```
In this version of slog, we will enforece a join order for nested facts. If no `?` operator is used in the body, join order of a nested clause will start from outer relation to nested inner relation. For example is previous example, join order will be `foobar` -> `foo` -> `bar`. If `?` operator is used, join order will start from inner relation to outer relation. For example:

```rust
slog! {
    // ...
    [(bar x y) <-- (foobar ?(foo x y) _) (bar x y)]
}
```
Join order will be `foo` -> `foobar` -> `bar`.

#### `!` Operator

`!` operator in original slog is used to fire a fact in body of a rule. In this version of slog, we will not support this operator as naive implementation will cause join order in generated Ascent code messed up.

### `nil`

`nil` is a special relation that is used to represent the empty set. It is used to represent the empty set in the head of a rule.
It is defalutly defined as `nil(1)` and `nil(0)` is the empty set and automatically added to all slog program.

```rust
slog! {
    (struct PathLength)
    (define path usize sexpr)
    (path 1 ?(nil 0))
}
```


### Equivalence (Coming Soon 🚧)

The first siginificant change required to support equivalence is allow unification operation as query to add union-find based relation. In ascent, this can be done via BYODS extension, however directly compile slog to slog-byods will cause readability issue in generated ascent code, which cause macro code too hard to debug. To make the generated ascent code simpler, we add some special syntax sugar to ascent to support unification as clause.

By adding annotate the ascent program with `#![egglog_mode]`, we can enable a ascent program natively equipment with an global hidden union-find relation.

```rust
ascent! {
    #![egglog_mode]
    struct EquivTest;

    relation ID foo(usize);
    relation ID bar(usize);

    foo(1);
    bar(2);

    inflated_a <=> b <-- foo(a), a <=> inflated_a, bar(b), b <=>! rep_b;
}
```
After egglog mode is enabled, we can use equivalence clause `<=>` and `<=>?` to add tuple and access the hidden union-find relation. Clause `<=>` is used to add two value into equivalence relation, and `<=>?` is used to fetch the representative of a given value in union-find relation.

Manually inflate each value to its equivalence relation during computation and shrink during unification can be tedious to write in plain ascent. In slog we allow automatic inflation and shrinking of values by declaring column of a relation as eclass.

```rust
slog! {
    (struct EClassTest)

    (define foo usize)
    (define bar usize)
    (define foobar eclass eclass)
    (define res eclass)

    (foobar (foo 1) (bar 1))
    [(union foo1 bar1) <-- (= foo1 (foo x)) (= bar1 (bar x))]
    // (rewrite! (foo x) (bar x))

    [(res x) <-- (foobar x x)]
}
```
Same as egglog, `union` can be used to declare equivalence of two tuple's identifier. In egglog, you have an extra syntax sugar `rewrite`, which is also supported in slog (I use `rewrite!` to futher distinguish from a normal fact rule).

> Known Issue: You can union any two `usize` value, but `union` on none eclass id value is undefined behavior.




