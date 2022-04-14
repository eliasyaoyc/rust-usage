use std::iter::Map;
use proc_macro2::{Ident, TokenStream};
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Path, Type, TypePath};
use syn::punctuated::{Iter, Punctuated};
use syn::token::{Comma, Token};
use quote::quote;

type TokenStreamIter<'a> = Map<Iter<'a, Field>, fn(&'a Field) -> TokenStream>;

pub struct BuilderContext {
    name: Ident,
    fields: Punctuated<Field, Comma>,
}

impl BuilderContext {
    pub fn new(input: DeriveInput) -> Self {
        let name = input.ident;
        let fields = if let Data::Struct(DataStruct {
                                             fields: Fields::Named(FieldsNamed { named, .. }), ..
                                         })
        = input.data
        {
            named
        } else {
            panic!("Unsupported data type");
        };
        Self { name, fields }
    }

    pub fn generate(&self) -> TokenStream {
        let name = &self.name;
        // builder name: {}Builder, e.g. CommandBuilder
        let builder_name = Ident::new(&format!("{}Builder", name), name.span());
        // optional fields. e.g. executable: String -> executable: Option<String>,
        let optionized_fields = self.gen_optionized_fields();
        // methods: fn executable(mut self, v: impl Into<String>) -> Self { self.executable = Some(v); self }
        // Command::builder().executable("hello").args(vec![]).envs(vec![]).finish()
        let methods = self.gen_methods();
        // assign Builder fields back to original struct fields
        // #field_name: self.#field_name.take().ok_or("xxx need to be set!")
        let assigns = self.gen_assigns();
        let ast = quote!(
            /// Builder structure
            #[derive(Debug,Default)]
            struct #builder_name {
                #(#optionized_fields,)*
            }

            impl #builder_name {
                #(#methods)*

                pub fn finish(mut self) -> Result<#name,&'static str> {
                   Ok(#name{
                      #(#assigns,)*
                   })
                }
            }

            impl #name {
                fn builder() -> #builder_name {
                    Default::default()
                }
            }
        );
        ast.into()
    }

    fn gen_optionized_fields(&self) -> TokenStreamIter {
        self.fields.iter().map(|f| {
            let (_, ty) = get_option_inner(&f.ty);
            let name = &f.ident;
            quote! { #name: std::option::Option<#ty>}
        })
    }

    fn gen_methods(&self) -> TokenStreamIter {
        self.fields.iter().map(|f| {
            let (_, ty) = get_option_inner(&f.ty);
            let name = &f.ident;
            quote! {
                pub fn #name(mut self, v: impl Into<#ty>) -> Self {
                    self.#name = Some(v.into());
                    self
                }
            }
        })
    }

    fn gen_assigns(&self) -> TokenStreamIter {
        self.fields
            .iter()
            .map(|f| {
                let name = &f.ident;
                let (optional, _) = get_option_inner(&f.ty);
                if optional {
                    quote! {
                    #name: self.#name.take()
                }
                } else {
                    quote! {
                    #name: self.#name.take().ok_or(concat!(stringify!(#name)," needs to be set!"))?
                }
                }
            })
    }
}

fn get_option_inner(ty: &Type) -> (bool, &Type) {
    if let Type::Path(TypePath {
                          path: Path { segments, .. },
                          ..
                      }) = ty
    {
        if let Some(v) = segments.iter().next() {
            if v.ident == "Option" {
                let t = match &v.arguments {
                    syn::PathArguments::AngleBracketed(a) => match a.args.iter().next() {
                        Some(GenericArgument::Type(t)) => t,
                        _ => panic!("Not sure what to do with other GenericArgument"),
                    },
                    _ => panic!("Not sure what to do with other PathArgument"),
                };
                return (true, t);
            }
        }
    }
    return (false, ty);
}