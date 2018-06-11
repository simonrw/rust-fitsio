extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(FitsRow, attributes(fitsio))]
pub fn read_row(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = &input.ident;

    let mut tokens = Vec::new();

    match &input.data {
        &syn::Data::Struct(ref s) => match &s.fields {
            &syn::Fields::Named(ref fields) => for field in &fields.named {
                let ident = &field.ident.as_ref().unwrap();
                let ident_str = ident.to_string();
                if field.attrs.is_empty() {
                    tokens.push(quote! {
                        out.#ident = tbl.read_cell_value(fits_file, #ident_str, idx)?;
                    });
                } else {
                    for attr in &field.attrs {
                        match attr.interpret_meta() {
                            Some(syn::Meta::List(l)) => for entry in l.nested {
                                match entry {
                                    syn::NestedMeta::Meta(syn::Meta::NameValue(
                                        syn::MetaNameValue {
                                            ident: attr_ident,
                                            lit,
                                            ..
                                        },
                                    )) => {
                                        if attr_ident.to_string() != "colname" {
                                            continue;
                                        }

                                        match lit {
                                            syn::Lit::Str(ls) => {
                                                tokens.push(quote! {
                                                    out.#ident = tbl.read_cell_value(
                                                        fits_file,
                                                        #ls,
                                                        idx)?;
                                                });
                                            }
                                            _ => panic!(
                                                "Only #[fitsio(colname = \"...\")] is supported"
                                            ),
                                        }
                                    }
                                    _ => panic!("Only #[fitsio(colname = \"...\")] is supported"),
                                }
                            },
                            _ => panic!("Only #[fitsio(colname = \"...\")] is supported"),
                        }
                    }
                }
            },
            _ => panic!("Only #[fitsio(colname = \"...\")] is supported"),
        },
        _ => panic!("derive only possible for structs"),
    }

    let expanded = quote!{
        impl FitsRow for #name {
            fn from_table(
                tbl: &::fitsio::hdu::FitsHdu,
                fits_file: &mut ::fitsio::FitsFile, idx: usize) ->
                    ::fitsio::errors::Result<Self> where Self: Sized  {
                let mut out = Self::default();

                #(#tokens)*

                Ok(out)
            }
        }
    };
    expanded.into()
}
