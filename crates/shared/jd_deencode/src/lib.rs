use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(Deen)]
pub fn derive_enum_common(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Deen chỉ áp dụng cho enum"),
    };

    let unit_variants: Vec<_> = variants
        .iter()
        .filter(|v| v.fields.is_empty())
        .collect();

    let from_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            #name::#variant_name => stringify!(#variant_name).to_lowercase(),
        }
    });

    let display_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            #name::#variant_name => write!(f, "{}", stringify!(#variant_name).to_lowercase()),
        }
    });

    let decode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            _ if s == stringify!(#variant_name).to_lowercase() => Ok(#name::#variant_name),
        }
    });

    let encode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            #name::#variant_name => stringify!(#variant_name).to_lowercase(),
        }
    });

    let expanded = quote! {
        impl From<#name> for sea_query::Value {
            fn from(value: #name) -> Self {
                let s = match value {
                    #(#from_patterns)*
                    _ => return sea_query::Value::String(None),
                };
                sea_query::Value::String(Some(Box::new(s)))
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
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
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
                let s_ref: &str = &s;
                <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s_ref, buf)
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
