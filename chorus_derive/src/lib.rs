use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ChoreographyLocation)]
pub fn derive_choreography_location(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ChoreographyLocation for #ident {
            fn new() -> Self {
                Self
            }
            fn name() -> &'static str {
                stringify!(#ident)
            }
        }
        impl Clone for #ident {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl Copy for #ident {}
    };
    output.into()
}

#[proc_macro_derive(Superposition)]
pub fn derive_superposition(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = syn::parse_macro_input!(input as DeriveInput);

    // Get the name of the struct or enum
    let name = &input.ident;

    // Generate the implementation of the Superposition trait
    let expanded = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|field| &field.ident);
                quote! {
                    impl Superposition for #name {
                        fn remote() -> Self {
                            #name {
                                #( #field_names: <_ as Superposition>::remote(), )*
                            }
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len()).map(|_| {
                    quote! {
                        <_ as Superposition>::remote()
                    }
                });
                quote! {
                    impl Superposition for #name {
                        fn remote() -> Self {
                            #name(
                                #(#fields),*
                            )
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    impl Superposition for #name {
                        fn remote() -> Self {
                            #name
                        }
                    }
                }
            }
        },
        Data::Enum(_) => {
            quote! {
                compile_error!("Superposition cannot be derived automatically for enums");
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Superposition cannot be derived automatically for unions");
            }
        }
    };

    // Convert the generated tokens back into a TokenStream
    TokenStream::from(expanded)
}
