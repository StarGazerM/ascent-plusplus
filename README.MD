# Ascent++

This is a fork of Ascent, a rust based datalog dailect made by [Arash](https://github.com/s-arash)

# What's new in this ++?

## Head Tuple Generating Dependency

In most of datalog engine, all head clauses are independent, there generateing order of each head clause is arbitrary and doesn't affect the final result. For example in the following datalog program:

```rust
ascent! {
   relation foo(i32);
   relation bar1(i32);
   relation bar2(i32);

   foo(1);
   bar1(1);
   bar1(x), bar2(x) <-- foo(x);
}
```

The result will contain `bar2(1)`, because the generating of `bar1(1)` and `bar2(1)` are independent. However, in some case, we only want to generate `bar2(1)` when `bar1(1)` is generated. In this example, since `bar1(1)` is a fact, it won't be generated in the last rule, so `bar2(1)` won't be generated either. This can be realized using `!` head cluase in this fork:

```rust
   !bar1(1), bar2(1) <-- foo(1);
``` 
In this case, `bar2(1)` will not be generated.

## Materialization of tuple ID
In this branch, you can get current size of relation using `let <y> = !<x>(...)` in the head clause to get the total size of relation x when tuple `<x>(...)` is generated and then use it autoinc ID. For example, in the following datalog program:

```rust
ascent! {
   relation foo(i32);
   relation bar(i32);
   relation bar_id(i32, usize);

   foo(1);
   let id = !bar(x), bar_d(x, id) <-- foo(x);
}
```
The detail semantic of this syntax can be found in slog paper (called ∃! in the paper).
a syntax sugar for ID is also provide:
```rust
   >?id.bar(x) <-- foo(x);
```
To declare a relation with ID, you can use  below syntax:
```rust
   relation ID bar(i32);
```
to declare `bar` and `bar_id` at the same time.
If you want use id of `bar` relation more elegantly, without directly use `bar_id`, you can use below syntax:
```rust
   ... <-- bar(x).id;
```
Using these materialized ID you can get slog like nested facts:
```rust
ascent! {
   relation ID edge(i32, i32);
   relation ID path(i32, Tag);
   relation input(usize);
    // normal TC
   >?id.edge(x, y) <-- edge_raw(x, y);
   >?new_id.path(x, nest_id.clone()) <--
      edge(x, y).eid,
      let nest_id = Tag("edge", *eid);
   >?new_id.path(x, nest_id.clone()) <--
      edge(x, y),
      path(y, _).pid,
      let nest_id = Tag("path", *pid);
}
```

## Defunctionalization
In slog, a function can be represented by relation with a nested "do" rule. For example to compute 
```scheme
(lenghth ?(do-length (edge x y)) 1)
[(length ?(do-length (path x y)) {+ l 1 })
    <--
    (edge x y),
    (length !(do-length y) l)]

(input (path 1 (path 2 (edge 3 y))))
```

In this branch, you can define do relation together with the relation using `function ` keyword. For example:
```rust
function path_length(Tag) -> usize;
```
both `path_length` and `do_path_length` will all paired with a id rule for id referencing.
A syntax sugar for referencing the do relation  without using actual function relation is:
```rust
   %path(x, y) -> ?;
```
It can be both used in head and body clause. For example:
```rust
   %path_length(Tag("path", *pid)) -> ? <--
      input(pid);
```
If `?` is repalced by a logical variable, it will also use the actual result of the function rule.

Using all these feature, above slog rule can be implemented as:
```rust
ascent! {
   struct Length;
   relation edge_raw(i32, i32);
   relation ID edge(i32, i32);
   relation ID path(i32, Tag);
   relation input(usize);
   // normal TC
   >?id.edge(x, y) <-- edge_raw(x, y);
   >?new_id.path(x, nest_id.clone()) <--
      edge(x, y).eid,
      let nest_id = Tag("edge", *eid);
   >?new_id.path(x, nest_id.clone()) <--
      edge(x, y),
      path(y, _).pid,
      let nest_id = Tag("path", *pid);

   function path_length(Tag) -> usize;
   // length of a path

   %path_length(Tag("path", *pid)) -> ?
        <--
        input(pid);

   %path_length(Tag("edge", *eid)) -> ret_val
        <--
        let ret_val = 1;

   %path_length(Tag("path", *pid)) -> ret_val
      <-- 
      path(x, res).pid,
      %path_length(res) -> rest_length,
      let ret_val = rest_length + 1;
}
```

## More powerful macro

Now you can use `va_list` to match the rest of unmatched variable in a macro definition. For example:
```rust
ascent! {
   macro exists($rel_name: ident, $id: ident, $args: va_list) {
        let $id = !$rel_name($args), $($rel_name)_id($args, $id)
   }
   exists!(edge, id, x, y) <-- edge_raw(x, y);
}
```
You can also use macro to generate relation definition and rules, but when calling these macro, you need a `@` before the callsite. For example:
```rust
ascent! {
   macro declare_id_rel($rel_name: ident, $args: va_list) {
        relation $rel_name($args);
        relation $($rel_name)_id($args, usize);
   }

   @declare_id_rel!(edge, i32, i32);
   @declare_id_rel!(path, i32, Tag);
}
```

## Derivation Counter
*WARNING: This can't been turned off in current code!*
*WARNING: This feature only works in you are using native relation(not lattice or BYODS relation)*
In this branch, tuple in full version of relation (in a ascent they are relation index with all of its columns) will have a tag associated with it. This tag is used to track the derivation of the tuple. For example, in the following datalog program:
```rust
fn test_tc() {
    let mut tc = ascent_run! {
        relation edge(i32, i32);
        relation path(i32, i32);

        edge(1, 2);
        edge(2, 3);
        edge(3, 4);
        edge(1, 4);

        path(x, y) <-- edge(x, y);
        path(x, y) <-- path(x, z), edge(z, y);
    };

    let path_with_cnt = tc.path_indices_0_1.iter().collect::<Vec<_>>();
    println!("path: {:?}", path_with_cnt);
    // path: [((1, 2), FullRelCounter { counter: 1 }), ((2, 4), FullRelCounter { counter: 1 }), 
    // ((3, 4), FullRelCounter { counter: 1 }), ((2, 3), FullRelCounter { counter: 1 }), ((1, 3),
    // FullRelCounter { counter: 1 }), ((1, 4), FullRelCounter { counter: 2 })]
}
```
The result will contain `path(1, 4)` with a counter of 2, because it is derived from two different paths.

