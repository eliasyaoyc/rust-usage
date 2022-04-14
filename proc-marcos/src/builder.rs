use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Path, Type,
    TypePath,
};


struct Fd {
    name: Ident,
    ty: Type,
    optional: bool,
}

pub struct BuilderContext {
    name: Ident,
    fields: Vec<Fd>,
}

impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        let (optional, ty) = get_option_inner(&f.ty);
        Self {
            name: f.ident.unwrap(),
            ty: ty.to_owned(),
            optional,
        }
    }
}

impl From<DeriveInput> for BuilderContext {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;
        let fields = if let Data::Struct(DataStruct {
                                             fields: Fields::Named(FieldsNamed { named, .. }),
                                             ..
                                         }) = input.data
        {
            named
        } else {
            panic!("Unsupported data type");
        };
        let fds = fields.into_iter().map(Fd::from).collect();
        Self { name, fields: fds }
    }
}

impl BuilderContext {
    pub fn render(&self) -> TokenStream {
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
        quote!(
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
        )
    }

    fn gen_optionized_fields(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, ty, .. }| quote! { #name: std::option::Option<#ty>})
            .collect()
    }

    fn gen_methods(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, ty, .. }| {
                quote! {
                pub fn #name(mut self, v: impl Into<#ty>) -> Self {
                    self.#name = Some(v.into());
                    self
                }
            }
            })
            .collect()
    }

    fn gen_assigns(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, optional, .. }| {
                if *optional {
                    return quote! {
                        #name: self.#name.take()
                    };
                }

                quote! {
                    #name: self.#name.take().ok_or(concat!(stringify!(#name), " needs to be set!"))?
                }
            })
            .collect()
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