use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ChoreographyLocation)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ChoreographyLocation for #ident {
            fn name(&self) -> &'static str {
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
