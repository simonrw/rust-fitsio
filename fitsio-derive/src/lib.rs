extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(FitsRow)]
pub fn read_row(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let expanded = impl_read_row(input);
    expanded.into()
}

fn impl_read_row(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;

    let mut tokens = Vec::new();

    match &input.data {
        &syn::Data::Struct(ref s) => match &s.fields {
            &syn::Fields::Named(ref fields) => for field in &fields.named {
                let ident = &field.ident.unwrap();
                let ident_str = ident.to_string();
                tokens.push(quote! {
                    out.#ident = tbl.read_cell_value(fits_file, #ident_str, idx)?;
                });
            },
            _ => unimplemented!(),
        },
        _ => panic!("derive only possible for structs"),
    }

    quote!{
        impl FitsRow for #name {
            fn from_table(tbl: &FitsHdu, fits_file: &mut FitsFile, idx: usize) -> Result<Self> where Self: Sized  {
                fits_file.make_current(tbl)?;
                let mut out = Self::default();

                #(#tokens)*

                Ok(out)
            }
        }
    }
}
