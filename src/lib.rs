#![no_std]

extern crate alloc;
extern crate core;
extern crate proc_macro;

#[allow(unused_imports)]
use libc_print::std_name::*;
use {
    alloc::string::ToString,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{ToTokens, quote},
    syn::{Data, DeriveInput, Fields, Ident, parse_macro_input}
};

#[proc_macro_derive(FromMap)]
pub fn derive_iterable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let struct_generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let fields = match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields_named) => fields_named.named,
                _ => panic!("Only structs with named fields are supported")
            }
        },
        _ => panic!("Only structs are supported")
    };

    let fields_iter = fields.iter().map(|field| {
        let field_type = field.ty.to_token_stream().to_string();
        let field_name = field.ident.as_ref().unwrap().to_string();

        quote! {
            (#field_name, #field_type)
        }
    });

    let fields_set = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = field.ty.to_token_stream().to_string();
        let field_value = match field_type
            .as_str()
            .trim_start_matches("Option ")
            .trim_start_matches("Box ")
            .trim_start_matches("Arc ")
            .trim_start_matches("Rc ")
            .trim_start_matches("RefCell ")
            .trim_start_matches("Cell ")
            .trim_matches(['<', '>', ' '])
        {
            ty @ ("bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32"
            | "u64" | "f32" | "f64") => {
                let ty = Ident::new(ty, Span::call_site());
                quote! { v.parse::<#ty>().unwrap().into() }
            },
            "char" => quote! { v.chars().next().unwrap_or_default().into() },
            "str" => quote! { v.into() },
            "String" => quote! { v.to_string().into() },
            sty if sty.starts_with('[') && sty.ends_with(']') => {
                let ty = Ident::new(sty.trim_matches(['[', ']']), Span::call_site());
                quote! { v.chars().map(|c| c as #ty).collect() }
            },
            _ => quote! { v.into() }
        };

        quote! {
            if let Some(Some(mut v)) = map.remove(#field_name).take() {
                v = v.trim();
                if v.is_empty() == false {
                    this.#field_ident = #field_value;
                }
            }
        }
    });

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            fn struct_fields<'iter>() -> alloc::vec::IntoIter<(&'iter str, &'iter str)> {
                vec![#(#fields_iter),*].into_iter()
            }
        }

        impl<'iter> #impl_generics FromIterator<(&'iter str, Option<&'iter str>)> for #struct_name #ty_generics #where_clause {
            fn from_iter<T: IntoIterator<Item = (&'iter str, Option<&'iter str>)>>(iter: T) -> Self {
                let mut this: Self = Default::default();
                let mut map = alloc::collections::BTreeMap::from_iter(iter.into_iter());

                dbg!(&map);

                #(#fields_set)*

                this
            }
        }
    };

    //println!("{}", expanded.to_string());

    TokenStream::from(expanded)
}
