#![cfg(test)]
use petgraph::dot::{Config, Dot};
use proc_macro2::TokenStream;

use crate::ascent_impl;


#[test]
fn test_macro0() {
   let inp = quote!{
      struct Polonius<T: FactTypes>;
      relation subset(T::Origin, T::Origin, T::Point);// = ctx.subset_base.clone();
      relation cfg_edge(T::Point, T::Point);
      relation origin_live_on_entry(T::Origin, T::Point);
      relation origin_contains_loan_on_entry(T::Origin, T::Loan, T::Point);
      relation loan_live_at(T::Loan, T::Point);
      relation loan_invalidated_at(T::Loan, T::Point);
      relation errors(T::Loan, T::Point);
      relation placeholder_origin(T::Origin);
      relation subset_error(T::Origin, T::Origin, T::Point);
      relation loan_killed_at(T::Loan, T::Point);// = loan_killed_at.iter().cloned().collect();
      relation known_placeholder_subset(T::Origin, T::Origin);// = known_placeholder_subset.iter().cloned().collect();

      subset(origin1, origin3, point) <--
         subset(origin1, origin2, point),
         subset(origin2, origin3, point),
         if origin1 != origin3;

      subset(origin1, origin2, point2) <--
         subset(origin1, origin2, point1),
         cfg_edge(point1, point2),
         origin_live_on_entry(origin1, point2),
         origin_live_on_entry(origin2, point2);

      origin_contains_loan_on_entry(origin2, loan, point) <--
         origin_contains_loan_on_entry(origin1, loan, point),
         subset(origin1, origin2, point);

      origin_contains_loan_on_entry(origin, loan, point2) <--
         origin_contains_loan_on_entry(origin, loan, point1),
         cfg_edge(point1, point2),
         !loan_killed_at(loan, point1),
         origin_live_on_entry(origin, point2);

      loan_live_at(loan, point) <--
         origin_contains_loan_on_entry(origin, loan, point),
         origin_live_on_entry(origin, point);

      errors(loan, point) <--
         loan_invalidated_at(loan, point),
         loan_live_at(loan, point);

      subset_error(origin1, origin2, point) <--
         subset(origin1, origin2, point),
         placeholder_origin(origin1),
         placeholder_origin(origin2),
         !known_placeholder_subset(origin1, origin2),
         if origin1 != origin2;
   };
   // write_ascent_run_to_scratchpad(inp);
   write_ascent_run_par_to_scratchpad(inp);
}
#[test]
fn test_macro_generic_tc() {
   let inp = quote!{
      #![ds(custom_ds)]
      struct TC<TNode> where TNode: Clone + std::cmp::Eq + std::hash::Hash + Sync + Send;
      #[ds(ascent::rel)]
      relation edge(TNode, TNode);
      relation path(TNode, TNode);

      path(x, z) <-- edge(x, y), path(y, z);
      // path(x, z) <-- path(x, y), path(y, z);
   };

   // write_to_scratchpad(inp);
   write_par_to_scratchpad(inp);
}

#[test]
fn test_macro_multiple_dynamic_clauses() {
   let inp = quote! {
      relation a(i32, i32);
      relation b(i32, i32);
      relation c(i32, i32);

      a(y, z),
      b(z, w),
      c(x, y) <--
         a(x, y),
         b(y, z),
         c(z, w);
   };
   write_to_scratchpad(inp);
}

#[test]
fn test_macro_tc() {
   let inp = quote!{
      // #![measure_rule_times]
      struct TC;
      relation edge(i32, i32);
      relation path(i32, i32);

      path(x, y) <-- edge(x, y);
      // path(x, z) <-- edge(x, y), path(y, z);
      path(x, z) <-- path(x, y), path(y, z);
   };

   // write_to_scratchpad(inp);
   write_par_to_scratchpad(inp);
}

#[test]
fn test_macro2() {
   let input = quote! {
      relation foo(i32, Option<i32>);
      relation bar(i32, i32);
      relation baz(i32, i32, i32);
      foo(1, Some(2));
      foo(2, None);
      foo(3, Some(5));
      foo(4, Some(10));


      bar(3, 6);
      bar(5, 10);
      bar(10, 20);

      baz(*x, *y, *z) <-- foo(x, ?Some(y)), bar(y , z);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_clone_warning() {
   let input = quote! {
      struct Ancestory<'a>;
      relation parent(&'a str, &'a str);
      relation ancestor(&'a str,&'a str);

      ancestor(p, c) <-- parent(p, c);

      ancestor(p, gc) <--
         parent(p, c), ancestor(c, gc);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_macro_unary_rels() {
   let input = quote! {
      relation foo(i32);
      relation bar(i32);
      relation baz(i32, i32);
      foo(1);

      bar(3);

      foo(x), bar(y) <-- baz(x, y);
      baz(x, x + 1) <-- foo(x), bar(x);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_macro_dep_head() {
   let input = quote! {
      relation foo(i32, i32);
      relation bar(i32, i32);
      relation foobar(i32, usize);

      foo(1, 2);
      bar(1, 2);

      let new_bar = !bar(x, y), foobar(x, new_bar) <-- foo(x, y);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_relation_id() {
   let input = quote! {
      relation foo(i32, i32);
      relation bar(i32, i32);
      relation foobar(i32, usize);

      foo(1, 2);

      let new_bar = !bar(x, y), foobar(x, new_bar) <-- foo(x, y);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_macro3() {
   let input = quote! {
      relation bar(i32, i32);
      relation foo(i32, i32);
      relation baz(i32, i32);

      foo(1, 2);
      foo(10, 2);
      bar(2, 3);
      bar(2, 1);

      baz(*x, *z) <-- foo(x, y) if *x != 10, bar(y, z), if x != z;
      foo(*x, *y), bar(*x, *y) <-- baz(x, y);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_macro_agg() {
   let inp = quote! {
      relation foo(i32);
      relation bar(i32, i32, i32);
      lattice baz(i32, i32);

      baz(x, min_z) <--
         foo(x),
         agg min_z = min(z) in bar(x, _, z);
   };
   write_par_to_scratchpad(inp);
}

#[test]
fn test_macro_generator() {
   let input = quote! {
      relation edge(i32, i32);
      relation path(i32, i32);
      edge(x, x + 1) <-- for x in 0..100;
      path(*x, *y) <-- edge(x,y);
      path(*x, *z) <-- edge(x,y), path(y, z);
   };

   write_par_to_scratchpad(input);
}

#[test]
fn test_macro_patterns() {
   let input = quote! {
      relation foo(i32, Option<i32>);
      relation bar(i32, i32);
      foo(1, None);
      foo(2, Some(2));
      foo(3, Some(30));
      bar(*x, *y) <-- foo(x, ?Some(y)) if y != x;
      bar(*x, *y) <-- foo(x, y_opt) if let Some(y) = y_opt if y != x;
   };

   write_to_scratchpad(input);
}

#[test]
fn test_macro_sp(){
   let input = quote!{
      relation edge(i32, i32, u32);
      lattice shortest_path(i32, i32, Dual<u32>);
      
      edge(1, 2, 30);

      shortest_path(x, y, Dual(*len)) <-- edge(x, y, len);
      shortest_path(x, z, Dual(len + plen)) <-- edge(x, y, len), shortest_path(y, z, ?Dual(plen));
   };
   // write_to_scratchpad(input);
   write_par_to_scratchpad(input);
}

#[test]
fn test_lattice(){
   let input = quote! {
      relation foo(i32, i32);
      relation bar(i32, i32);

      bar(x, x+1) <-- for x in 0..10;
      foo(*x, *y) <-- bar(x, y);

      lattice foo_as_set(ascent::lattice::set::Set<(i32, i32)>);
      foo_as_set(ascent::lattice::set::Set::singleton((*x, *y))) <-- foo(x, y);

      relation baz(i32, i32);
      baz(1, 2);
      baz(1, 3);

      relation res(i32, i32);
      res(*x, *y) <-- baz(x, y), foo_as_set(all_foos), if !all_foos.contains(&(*x, *y));
   };
   write_to_scratchpad(input);
}

#[test]
fn test_macro_lattices(){
   let input = quote!{
      lattice longest_path(i32, i32, u32);
      relation edge(i32, i32, u32);

      longest_path(x, y, ew) <-- edge(x, y, ew);
      // longest_path(x, z, *ew + *w) <-- edge(x, y, ew), longest_path(y, z, w);
      longest_path(x, z, *l1 + *l2) <-- longest_path(x, y, l1), longest_path(y, z, l2);


      // edge(1,2, 3);
      // edge(2,3, 5);
      // edge(1,3, 4);
      // edge(2,4, 10);

   };
   // write_to_scratchpad(input);
   write_par_to_scratchpad(input);
}

#[test]
fn test_simple() {
   let input = quote! {
      relation bar(i32, i32);
      relation foo(i32, i32);
      relation baz(i32, i32);

      foo(1, 2);
      foo(10, 2);
      bar(2, 3);
      bar(2, 1);

      baz(*x, *z) <-- foo(x, y) if *x != 10, bar(y, ?z) if *z < 4, if x != z;
      
      baz(*x, *z) <-- foo(x, y) if *x != 10, bar(y, z) if *z * x != 444, if x != z;
      foo(*x, *y), bar(*x, *y) <-- baz(x, y);
   };

   write_to_scratchpad(input);
}

#[test]
fn test_no_generic(){
   let input = quote!{
      struct AscentProgram;
      relation dummy(usize);
   };
   // write_to_scratchpad(input);
   write_to_scratchpad(input);
}

#[test]
fn test_generic_ty(){
   let input = quote!{
      struct AscentProgram<T: Clone + Hash + Eq>;
      relation dummy(T);
   };
   // write_to_scratchpad(input);
   write_to_scratchpad(input);
}

#[test]
fn test_generic_ty_where_clause(){
   let input = quote!{
      struct AscentProgram<T> where T: Clone + Hash + Eq;
      relation dummy(T);
   };
   write_to_scratchpad(input);
}

#[test]
fn test_generic_ty_with_divergent_impl_generics(){
   let input = quote!{
      struct AscentProgram<T>;
      impl<T: Clone + Hash + Eq> AscentProgram<T>;
      relation dummy(T);
   };
   write_to_scratchpad(input);
}

#[test]
fn test_generic_ty_with_divergent_impl_generics_where_clause(){
   let input = quote!{
      /// Type DOC COMMENT
      struct AscentProgram<T>;
      impl<T> AscentProgram<T> where T: Clone + Hash + Eq;
      /// dummy REL DOC COMEMNT
      relation dummy(T);
   };
   write_to_scratchpad(input);
}

#[test]
fn exp_borrowing(){
   // let mut v: Vec<i32> = vec![];
   // let mut u: Vec<i32> = vec![];
   // for i in 0..v.len(){
   //    let v_row = &v[i];

   //    for j in 0..u.len(){
   //       let u_row = &u[j];
   //       let new_row = *u_row + *v_row;
   //       v.push(new_row);
   //    }
   // }

   // let x: Vec<i32> = vec![42];
   // let y: Vec<i32> = Convert::convert(&x);
   // let z: Vec<i32> = Convert::convert(x);
}

#[test]
fn exp_condensation() {
   use petgraph::algo::condensation;
   use petgraph::prelude::*;
   use petgraph::Graph;

   let mut graph: Graph<&'static str, (), Directed> = Graph::new();
   let a = graph.add_node("a"); // node with no weight
   let b = graph.add_node("b");
   let c = graph.add_node("c");
   let d = graph.add_node("d");
   let e = graph.add_node("e");
   let f = graph.add_node("f");
   let g = graph.add_node("g");
   let h = graph.add_node("h");

   // a ----> b ----> e ----> f
   // ^       |       ^       |
   // |       v       |       v
   // d <---- c       h <---- g
   graph.extend_with_edges(&[(a, b), (b, c), (c, d), (d, a), (b, e), (e, f), (f, g), (g, h), (h, e)]);
   let acyclic_condensed_graph = condensation(graph.clone(), true);
   #[allow(non_snake_case)]
   let (A, B) = (NodeIndex::new(0), NodeIndex::new(1));
   assert_eq!(acyclic_condensed_graph.node_count(), 2);
   assert_eq!(acyclic_condensed_graph.edge_count(), 1);
   assert_eq!(acyclic_condensed_graph.neighbors(B).collect::<Vec<_>>(), vec![A]);

   println!("{:?}", Dot::with_config(&acyclic_condensed_graph, &[Config::EdgeNoLabel]));

   let sccs = petgraph::algo::tarjan_scc(&graph);
   println!("sccs ordered:");
   for scc in sccs.iter(){
      println!("{:?}", scc);
   }
}

#[test]
fn exp_items_in_fn(){
   let mut p = Default::default();
   for i in 0..10 {
      p = {
         #[derive(Debug, Default)]
         struct Point{x: i32, y: i32}
         impl Point {
            pub fn size(&self) -> i32 {self.x * self.x + self.y * self.y}
         }
         Point{x:i, y: i+1}
      };
   }
   println!("point is {:?}, with size {}", p, p.size());
}

fn write_to_scratchpad_base(tokens: TokenStream, prefix: TokenStream, is_ascent_run: bool, is_parallel: bool) -> TokenStream {
   let code = ascent_impl(tokens, is_ascent_run, is_parallel).unwrap();
   let code = quote! {
      #prefix
      #code
   };
   let template = std::fs::read_to_string("src/scratchpad_template.rs").unwrap();
   let code_in_template = template.replace("todo!(\"here\");", &code.to_string());
   std::fs::write("src/scratchpad.rs", prefix.to_string()).unwrap();
   std::fs::write("src/scratchpad.rs", code_in_template).unwrap();
   std::process::Command::new("rustfmt").args(&["src/scratchpad.rs"]).spawn().unwrap().wait().unwrap();
   code
}

fn write_to_scratchpad(tokens: TokenStream) -> TokenStream {
   write_to_scratchpad_base(tokens, quote!{}, false, false)
}

fn write_duo_to_scratchpad(tokens1: TokenStream, tokens2: TokenStream) -> TokenStream {
   let p2 = ascent_impl(tokens2, false, false).unwrap();
   write_to_scratchpad_base(tokens1, p2, false, false)
}

fn write_with_prefix_to_scratchpad(tokens: TokenStream, prefix: TokenStream) -> TokenStream {
   write_to_scratchpad_base(tokens, prefix, false, false)
}

fn write_par_to_scratchpad(tokens: TokenStream) -> TokenStream {
   write_to_scratchpad_base(tokens, quote!{}, false, true)
}

fn write_duo_par_to_scratchpad(tokens1: TokenStream, tokens2: TokenStream) -> TokenStream {
   let p2 = ascent_impl(tokens2, false, true).unwrap();
   write_to_scratchpad_base(tokens1, p2, false, true)
}


#[allow(unused)]
fn write_ascent_run_to_scratchpad(tokens: TokenStream) -> TokenStream {
   write_to_scratchpad_base(tokens, quote!{}, true, false)
}

fn write_ascent_run_par_to_scratchpad(tokens: TokenStream) -> TokenStream {
   write_to_scratchpad_base(tokens, quote!{}, true, true)
}


#[test]
fn test_macro_lambda_calc(){
   let prefix = quote! {
      #[derive(Clone, PartialEq, Eq, Debug, Hash)]
      pub enum LambdaCalcExpr{
         Ref(&'static str),
         Lam(&'static str, Rc<LambdaCalcExpr>),
         App(Rc<LambdaCalcExpr>, Rc<LambdaCalcExpr>)
      }

      use LambdaCalcExpr::*;

      impl LambdaCalcExpr {
         #[allow(dead_code)]
         fn depth(&self) -> usize {
            match self{
               LambdaCalcExpr::Ref(_) => 0,
               LambdaCalcExpr::Lam(_x,b) => 1 + b.depth(),
               LambdaCalcExpr::App(f,e) => 1 + max(f.depth(), e.depth())
            }
         }
      }
      fn app(f: LambdaCalcExpr, a: LambdaCalcExpr) -> LambdaCalcExpr {
         App(Rc::new(f), Rc::new(a))
      }
      fn lam(x: &'static str, e: LambdaCalcExpr) -> LambdaCalcExpr {
         Lam(x, Rc::new(e))
      }

      fn sub(exp: &LambdaCalcExpr, var: &str, e: &LambdaCalcExpr) -> LambdaCalcExpr {
         match exp {
            Ref(x) if *x == var => e.clone(),
            Ref(_x) => exp.clone(),
            App(ef,ea) => app(sub(ef, var, e), sub(ea, var, e)),
            Lam(x, _eb) if *x == var => exp.clone(),
            Lam(x, eb) => lam(x, sub(eb, var, e))
         }
      }

      #[allow(non_snake_case)]
      fn U() -> LambdaCalcExpr {lam("x", app(Ref("x"), Ref("x")))}
      #[allow(non_snake_case)]
      fn I() -> LambdaCalcExpr {lam("x", Ref("x"))}

      fn min<'a>(inp: impl Iterator<Item = (&'a i32,)>) -> impl Iterator<Item = i32> {
         inp.map(|tuple| tuple.0).min().cloned().into_iter()
      }
   };
   let inp = quote!{
      relation output(LambdaCalcExpr);
      relation input(LambdaCalcExpr);
      relation do_eval(LambdaCalcExpr);
      relation eval(LambdaCalcExpr, LambdaCalcExpr);

      input(app(U(), I()));
      do_eval(exp.clone()) <-- input(exp);
      output(res.clone()) <-- input(exp), eval(exp, res);

      eval(exp.clone(), exp.clone()) <-- do_eval(?exp @Ref(_));

      eval(exp.clone(), exp.clone()) <-- do_eval(exp), if let Lam(_,_) = exp;

      do_eval(ef.as_ref().clone()) <-- do_eval(?App(ef,_ea));

      do_eval(sub(fb, fx, ea)) <-- 
         do_eval(?App(ef, ea)), 
         eval(ef.deref(), ?Lam(fx, fb));
      
      eval(exp.clone(), final_res.clone()) <-- 
         do_eval(?exp @ App(ef, ea)), // this requires nightly
         eval(ef.deref(), ?Lam(fx, fb)),
         eval(sub(fb, fx, ea), final_res);
   };
   write_with_prefix_to_scratchpad(inp, prefix);
}

#[test]
fn test_macro_in_macro() {
   let inp = quote!{
      relation foo(i32, i32);
      relation bar(i32, i32);

      macro foo_($x: expr, $y: expr) {
         foo($x, $y)
      }

      macro foo($x: expr, $y: expr) {
         let _x = $x, let _y = $y, foo_!(_x, _y)
      }

      foo(0, 1);
      foo(1, 2);
      foo(2, 3);
      foo(3, 4);

      bar(x, y) <-- foo(x, y), foo!(x + 1, y + 1), foo!(x + 2, y + 2), foo!(x + 3, y + 3);
      
   };

   write_to_scratchpad(inp);
}

#[test]
fn test_function() {
   let prefix = quote! {
      #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
      struct Tag(&'static str, usize);
   };
   let inp = quote!{
      relation ID edge(i32, i32);
      relation ID path(i32, Tag);
      relation input(i32, i32);

      function path_length(Tag) -> usize;
      %path_length(?Tag("edge", pid)) -> ret_val
        <-- 
        path(x, res).*pid,
        %path_length(res) -> rest_length,
        let ret_val = rest_length + 1;
   };

   write_with_prefix_to_scratchpad(inp, prefix);
}

#[test]
fn test_macro_lattices_slow () {
   let inp = quote! {
      #![measure_rule_times]
      lattice longest_range(usize);
      longest_range(0);
      // relation input_string(usize, u8);
      // input_string(i, c) <-- for i in 1..s.len() + 1, let c = s.as_bytes()[i - 1];

      // the longest range ends at i is j, inclusive
      lattice longest_ranges_ends(usize, usize);

      // case 0: "" empty is a range has size 0 
      // init longest_range to 0
      longest_ranges_ends(i + 1, 0) <-- for i in 0..s.len();

      // case 1 : ( {range} ) is a range
      longest_ranges_ends(i + 1, j + 2) <--
         longest_ranges_ends(i, j),
         if i < &s.len() && i > j, 
         if s.as_bytes()[*i] == b')' && s.as_bytes()[i - j - 1] == b'(';
         // input_string(i + 1, b')'),
         // input_string(i - j, b'(');

      // case 2 : {range} {range} is a range
      // if two ranges are connected, update the later range's size
      longest_ranges_ends(i, size1 + size2) <--
         longest_ranges_ends(i, size1),
         longest_ranges_ends(i - size1, size2);

      longest_range(size) <-- longest_ranges_ends(_, size);
   };
   write_to_scratchpad(inp);
}

#[test]
fn test_macro_delta() {
   let inp = quote! {
      relation foo(i32, i32);
      relation bar(i32, i32);
      relation baz(i32, i32);
      relation foobar(i32, i32);

      foo(a, c) <-- bar(a, b), baz(b, c), bar(a, c);
      bar(a, c) <-- bar(a, b), baz(b, c), bar(a, c);
      baz(a, c) <-- bar(a, b), baz(b, c), bar(a, c);
   };

   write_par_to_scratchpad(inp);
}

#[test]
fn test_macro_incremental() {
   let inp = quote! {
      struct TCIncremental;
      relation edge_incremental(i32, i32);
      relation path_incremental(i32, i32);
      relation edge(i32, i32);
      relation path(i32, i32);
      relation outside();

      edge(x, y) <-- edge_incremental(x, y);
      path(x, y) <-- path_incremental(x, y);
      edge_incremental(x, y) <-- outside(), let x = 0, let y = 0;
      path_incremental(x, y) <-- outside(), let x = 0, let y = 0;

      path(x, y) <-- delta edge(x, y);
      path(x, z) <-- delta path(x, y), edge(y, z);

      path_incremental(x, y) <-- path(x, y);
      edge_incremental(x, y) <-- edge(x, y);
      outside() <-- edge(x, y);
      outside() <-- path(x, y);
   };

   write_par_to_scratchpad(inp);
}

#[test]
fn test_extern_database1() {
   let inp1 = quote! {
      struct TC;

      relation edge(i32, i32);
      relation path(i32, i32);
      
      index path ();
   };

   let inp2 = quote! {
      struct ExtTest;
      extern database TC tc();

      relation edge(i32, i32);
      relation path(i32, i32) in tc;
      
      edge(x, y) <-- tc.path(x, y);
      
   };

   write_duo_to_scratchpad(inp1, inp2);
}

#[test]
fn test_extern_database2() {
   let inp1 = quote! {
      struct Graph;

      relation edge(i32, i32);
      index edge (0, 1);
   };

   let inp2 = quote! {
      struct Printer;

      extern database Graph graph();
      relation edge(i32, i32) in graph;

      relation print(i32, i32);

      print(x, y) <-- print(x, y), graph.edge(y, x);
      
   };

   write_duo_par_to_scratchpad(inp1, inp2);
}

#[test]
fn test_nest_extern_database() {
   let inp1 = quote! {
      struct Graph;

      relation edge(i32, i32);
      relation test(i32, i32);
      index edge (1);
   };
   let inp2 = quote! {
      struct SSSPEager;

      extern database Graph graph();
      
      relation edge(i32, i32) in graph;
      relation test(i32, i32) in graph;

      relation do_length(i32, i32);

      lattice ret(i32);

      ret(1), graph.test(x, y) <-- do_length(x, y), graph.edge(x, y);
      
      ret(ret_val+1) <--
         do_length(x, y),
         do g : SSSPEager {
            do_length : vec![(*x, *y)]
         } (graph),
         let ret_val = g.ret[0].0;
   };
   write_duo_to_scratchpad(inp1, inp2);
}


#[test]
fn test_extern_arg() {
   let inp = quote! {
      struct Foo;

      extern arguement i32 test_foo_arg;

      relation foo(i32, i32);
      relation bar(i32);
      foo(x, test_foo_arg) <-- bar(x);
   };

   write_to_scratchpad(inp);
}

#[test]
fn test_io() {
   let inp = quote! {
      relation input(i32, i32);
      relation output(i32, i32);

      await input;
      yield output;

      input(1, 2);
      output(x, y) <-- input(x, y);
   };
   write_to_scratchpad(inp);
}

#[test]
fn test_run_timeout() {
   let input = quote! {
      #![generate_run_timeout]
      /// A diverging Ascent program
      struct Diverging;
      /// foooooooooooo
      relation foo(u128);
      foo(0);
      foo(x + 1) <-- foo(x);
   };
   write_to_scratchpad(input);
}

