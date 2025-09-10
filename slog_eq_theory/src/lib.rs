use quote::quote;

#[proc_macro]
pub fn canonicalize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let input = syn::parse_macro_input!(input as syn::Expr);
   let res = quote! {
       _self.runtime_total.__unify_ind_common.combined
           .get_dominant_elem(#input)
           .unwrap_or(_self.runtime_delta.__unify_ind_common.combined
               .get_dominant_elem(#input).unwrap_or(#input))
   };
   res.into()
}

#[proc_macro]
pub fn unification_ds(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let input = syn::parse_macro_input!(input as syn::Expr);
   let res = quote! {
       #[ds(#input)]
   };
   res.into()
}

#[proc_macro]
pub fn theory_propagation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let input = syn::parse_macro_input!(input as syn::Expr);
   let res = quote! {

      // congruence code
      unify(parent_a, parent_b) <--
         unify(child_a, child_b),
         deriv(parent_a, child_a),
         deriv(parent_b, child_b),
         agg sibs_a = collect(sib) in deriv(parent_a, sib),
         agg sibs_b = collect(sib) in deriv(parent_b, sib),
         if _self.runtime_total.__unify_ind_common.combined.equiv_vec_huh(&sibs_a, &sibs_b)
            || _self.runtime_delta.__unify_ind_common.combined.equiv_vec_huh(&sibs_a, &sibs_b)
           ;
   };
   res.into()
}
