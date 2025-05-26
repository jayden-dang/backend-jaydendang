use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

fn convert_to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = name.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }

    result
}

#[proc_macro_derive(Deen)]
pub fn derive_enum_common(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("only for Enum"),
    };

    let unit_variants: Vec<_> = variants.iter().filter(|v| v.fields.is_empty()).collect();

    // 🚀 Pre-compute type name at compile time
    let type_name = format!("{}_enum", convert_to_snake_case(&name_str));

    let from_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_str = convert_to_snake_case(&variant_name.to_string());
        quote! {
            #name::#variant_name => #variant_str,
        }
    });

    let display_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_str = convert_to_snake_case(&variant_name.to_string());
        quote! {
            #name::#variant_name => write!(f, #variant_str),
        }
    });

    let decode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_str = convert_to_snake_case(&variant_name.to_string());
        quote! {
            #variant_str => Ok(#name::#variant_name),
        }
    });

    let encode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_str = convert_to_snake_case(&variant_name.to_string());
        quote! {
            #name::#variant_name => #variant_str,
        }
    });

    let expanded = quote! {
        impl From<#name> for sea_query::Value {
            fn from(value: #name) -> Self {
                let s = match value {
                    #(#from_patterns)*
                    _ => return sea_query::Value::String(None),
                };
                // ✅ Use static string - no allocation
                sea_query::Value::String(Some(Box::new(s.to_string())))
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_patterns)*
                    _ => write!(f, "unknown"),
                }
            }
        }

        impl sqlx::Type<sqlx::Postgres> for #name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                // ✅ Use compile-time constant - zero runtime cost
                const TYPE_NAME: &'static str = #type_name;
                sqlx::postgres::PgTypeInfo::with_name(TYPE_NAME)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for #name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>
            ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                match s {
                    #(#decode_patterns)*
                    _ => Err(format!("Unknown {}: {}", stringify!(#name), s).into()),
                }
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for #name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer
            ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = match self {
                    #(#encode_patterns)*
                    _ => return Err("Complex enum variants are not supported for DB storage".into()),
                };
                // ✅ Direct string reference - minimal allocation
                <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
            }
        }

        impl sea_query::Nullable for #name {
            fn null() -> sea_query::Value {
                sea_query::Value::String(None)
            }
        }
    };

    TokenStream::from(expanded)
}
