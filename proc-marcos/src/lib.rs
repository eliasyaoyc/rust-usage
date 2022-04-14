extern crate core;

mod json_schema;
mod builder;
mod builder_with_attr;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
use crate::builder::BuilderContext;
use crate::json_schema::{get_string_literal, StructsTemplate};

#[proc_macro]
pub fn sql(_input: TokenStream) -> TokenStream {
    "fn hello() { println!(\"hello world\"); }".parse().unwrap()
}

#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let filename = get_string_literal(input).unwrap();
    let result = StructsTemplate::render(&filename).unwrap();
    println!("{}", result);
    result.parse().unwrap()
}

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder::BuilderContext::from(input).render().into()
}

#[proc_macro_derive(BuilderWithAttr, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder_with_attr::BuilderContext::from(input)
        .render()
        .into()
}
