#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;
extern crate proc_macro2;

use {
    alloc::string::ToString,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{quote, ToTokens},
    syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Lifetime, LifetimeParam}
};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;

#[proc_macro_derive(SetFromIter)]
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
        let field_name = field
            .ident
            .as_ref()
            .expect("Couldn't get ident of field")
            .to_string();

        quote! {
            (#field_name, #field_type)
        }
    });

    let fields_set = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field.ident.as_ref().expect("Couldn't get ident of field").to_string();

        if field_name.starts_with("_") {
            return quote! {};
        }

        let mut field_type = field.ty.to_token_stream().to_string();
        while let Some(p) = field_type.find("< '") {
            field_type.replace_range(p ..= p + field_type[p..].find('>').unwrap(), "");
        }
        let mut field_type_inner = field_type.get(
            field_type.rfind('<').map(|i| i+1).unwrap_or(0)
            ..field_type.find('>').unwrap_or(field_type.len())
        ).unwrap().trim();
        field_type_inner = field_type_inner.get(
            field_type_inner.rfind(' ').map(|i| i+1).unwrap_or(0)..
        ).unwrap().trim_matches(['[',']',' ']);

        let field_type_str = if field_type.contains("Vec <") || field_type.contains("[") {
            "Vec"
        } else {
            field_type_inner
        };

        //dbg!(&field_type, field_type_str, field_type_inner);

        let mut is_field_struct = false;

        let mut field_value = match field_type_str {
            ty @ ("bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32"
            | "u64" | "u128" | "f32" | "f64" | "f128" | "isize" | "usize" | "c_char" | "c_short" | "c_ushort"
            | "c_int" | "c_uint" | "c_long" | "c_ulong" | "c_longlong" | "c_ulonglong" | "c_double" | "c_float" ) => {
                let ident = Ident::new(ty, Span::call_site());
                quote! {
                    v.parse::<#ident>()
                        .map_err(|_| concat!("Failed parse '{v}' to type ", #field_type).replace("{v}", v))?
                }
            },
            "char" => quote! {v.chars().next().unwrap_or_default()},
            "str" => quote! {v},
            "String" => quote! { v.to_string() },
            ty @ "Vec" => {
                let ident = Ident::new(field_type_inner, Span::call_site());
                match field_type_inner{
                    "String" | "str" => quote! {
                        v.split_terminator(',')
                            .map(|s| s.trim().into())
                            .collect::<::alloc::vec::Vec<_>>()
                    },
                    _ => quote! {{
                        let mut arr = ::alloc::vec::Vec::new();
                        for s in v.split_terminator(',') {
                            arr.push(
                                s.trim()
                                    .parse::<#ident>()
                                    .map_err(|_| concat!("Failed parse '{s}' to type ", #ty).replace("{s}", s))?
                                    .into()
                            );
                        }
                        arr
                    }},
                }
            },
            ty if ty.contains([':','\'']) == false => {
                is_field_struct = true;
                quote! {{
                    let sub_map = map
                        .iter_mut()
                        .filter_map(|(name, value)| {
                            name.starts_with(concat!(#field_name, "."))
                                .then(|| (name.trim_start_matches(concat!(#field_name, ".")), value.take()))
                        });
                    sub_map
                }}
            },
            _ => panic!("Unsupported field type {field_type:?}")
        };

        for mut ty in field_type.as_str()[..field_type.rfind('<').unwrap_or(0)].rsplit("<") {
            ty = ty.trim();
            if ty.is_empty() == false {
                let type_ident = Ident::new(ty, Span::call_site());
                field_value = match ty {
                    "Option" | "NonNull" => quote! {#type_ident::from(#field_value)},
                    "Box" if field_type.contains("Box < str >") => quote! {#field_value.into()},
                    "Vec" => field_value,
                    _ => quote! {#type_ident::new(#field_value)}
                }
            }
        }

        if is_field_struct {
            quote! {
                if map.iter().any(|(name,..)| name.starts_with(concat!(#field_name,"."))) {
                    self.#field_ident.set_from_iter(#field_value)?;
                }
            }
        } else {
            quote! {
                if let Some(Some(mut v)) = map.remove(#field_name).take() {
                    v = v.trim();
                    if v.is_empty() == false {
                        self.#field_ident = #field_value;
                    }
                }
            }
        }
    });

    let mut lifetime = LifetimeParam::new(Lifetime::new("'iter", Span::call_site()));
    struct_generics
        .lifetimes()
        .for_each(|l| lifetime.bounds.push(l.lifetime.clone()));

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            fn struct_fields() -> &'static [(&'static str, &'static str)] {
                &[#(#fields_iter),*]
            }

            fn set_from_iter<#lifetime, I>(&mut self, iter: I) -> Result<(), ::alloc::boxed::Box<dyn ::core::error::Error>>
            where
                I: IntoIterator<Item = (&'iter str, Option<&'iter str>)>
            {
                let mut map = ::alloc::collections::BTreeMap::from_iter(iter.into_iter());
                //dbg!(&map);

                #(#fields_set)*

                Ok(())
            }
        }
    };

    //eprintln!("{}", expanded.to_string());

    TokenStream::from(expanded)
}
