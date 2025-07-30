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

We support the `!` and `?` syntax sugar from the original Slog implementation for fact manipulation and unification.

### Egglog-like Equivalence (Coming Soon 🚧)

We are actively working on supporting `egglog`-style e-graphs and theory inflation operations directly within Slog. Stay tuned!
