use proc_macro2::Span;
use rand::Rng;
use syn::Ident;

// create a new ident with the given name + a random suffix
pub fn new_ident(name: &str) -> Ident {
   let suffix = rand::rng().random_range(0..1000000);
   Ident::new(&format!("{}_{}", name, suffix), Span::call_site())
}
