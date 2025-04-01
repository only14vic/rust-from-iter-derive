#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;
extern crate proc_macro2;

use {
    alloc::string::ToString,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{quote, ToTokens},
    syn::{parse_macro_input, Data, DeriveInput, Fields, Ident}
};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;

#[proc_macro_derive(FromIter)]
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
        let field_type = field.ty.to_token_stream().to_string();
        let mut field_type_str = field_type.as_str();

        const TRIM_TYPES: [&str; 8] = ["Option <", "Box <", "Arc <", "Rc <", "RefCell <", "Cell <", "NonZero <", "NonNull <"];
        const TRIM_TYPE_SYMBOLS: [char; 4] = ['<', '>', ' ', '&'];
        const TRIM_TYPE_EXTRA_SYMBOLS: [char; 6] = ['<', '>', ' ', '&', '[', ']'];

        loop {
            for ty in TRIM_TYPES {
                field_type_str = field_type_str.trim_start_matches(ty).trim_matches(TRIM_TYPE_SYMBOLS);
            }
            if TRIM_TYPES.iter().any(|&ty| field_type_str.starts_with(ty)) == false {
                break;
            }
        }

        //dbg!(field_type_str);

        let mut is_field_struct = false;

        let mut field_value = match field_type_str {
            ty @ ("bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32"
            | "u64" | "u128" | "f32" | "f64" | "f128" | "isize" | "usize" | "c_char" | "c_short" | "c_ushort"
            | "c_int" | "c_uint" | "c_long" | "c_ulong" | "c_longlong" | "c_ulonglong" | "c_double" | "c_float" ) => {
                let ident = Ident::new(ty, Span::call_site());
                quote! {
                    v.parse::<#ident>()
                        .expect(&concat!("Failed parse '{v}' to type ", #field_type).replace("{v}", v))
                }
            },
            "char" => quote! {v.chars().next().unwrap_or_default()},
            "str" => quote! {v},
            "String" => quote! { v.to_string() },
            mut ty if ty.starts_with("Vec ") || ty.starts_with('[') && ty.ends_with(']') => {
                ty = ty.trim_start_matches("Vec");
                for prefix in TRIM_TYPES {
                    ty = ty.trim_start_matches(prefix).trim_matches(TRIM_TYPE_EXTRA_SYMBOLS);
                }
                let ident = Ident::new(ty, Span::call_site());
                match ty {
                    "String" | "str" => quote! {
                        v.split_terminator(',')
                            .map(|s| {
                                s.trim().into()
                            })
                            .collect::<Vec<_>>()
                    },
                    _ => quote! {
                        v.split_terminator(',')
                            .map(|s| {
                                s.trim()
                                    .parse::<#ident>()
                                    .expect(&concat!("Failed parse '{s}' to type ", #ty).replace("{s}", s))
                                    .into()
                            })
                            .collect::<Vec<_>>()
                    },
                }
            },
            ty if ty.contains(['<',':','\'']) == false => {
                is_field_struct = true;
                let ty = Ident::new(ty, Span::call_site());
                quote! {{
                    let sub_map = map
                        .iter_mut()
                        .filter_map(|(name, value)| {
                            name.starts_with(concat!(#field_name, "."))
                                .then(|| (name.trim_start_matches(concat!(#field_name, ".")), value.take()))
                        })
                        .collect::<alloc::vec::Vec<_>>();
                    #ty::from_iter(sub_map)
                }}
            },
            _ => panic!("Unsupported field type {field_type:?}")
        };

        for mut ty in field_type.as_str()[..field_type.rfind('<').unwrap_or(0)].rsplit("<") {
            ty = ty.trim();
            if ty.is_empty() == false {
                let type_ident = Ident::new(ty, Span::call_site());
                field_value = match ty {
                    "Option" => quote! {Option::from(#field_value)},
                    "Box" if field_type.contains("Box < str >") => quote! {#field_value.into()},
                    "Vec" => field_value,
                    _ => quote! {#type_ident::new(#field_value)}
                }
            }
        }

        if is_field_struct {
            quote! {
                if map.iter().any(|(name,..)| name.starts_with(concat!(#field_name,"."))) {
                    this.#field_ident = #field_value;
                }
            }
        } else {
            quote! {
                if let Some(Some(mut v)) = map.remove(#field_name).take() {
                    v = v.trim();
                    if v.is_empty() == false {
                        this.#field_ident = #field_value;
                    }
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

                //dbg!(&map);

                #(#fields_set)*

                this
            }
        }
    };

    //eprintln!("{}", expanded.to_string());

    TokenStream::from(expanded)
}
