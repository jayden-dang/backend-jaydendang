// jd_deencode/src/lib.rs
// This crate contains ONLY the proc-macro

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse2, parse_macro_input, punctuated::Punctuated, Attribute, Data, DataEnum,
    DeriveInput, Expr, ExprLit, Lit, Meta, MetaList, MetaNameValue, Token,
};

struct MetaListParser(Punctuated<Meta, Token![,]>);

impl Parse for MetaListParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(MetaListParser(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro_derive(Deen, attributes(deen))]
pub fn derive_enum_common(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Deen chỉ áp dụng cho enum"),
    };


    // Extract custom postgres type name from attributes
    let postgres_type = extract_postgres_type(&input.attrs).unwrap();

    let unit_variants: Vec<_> = variants.iter().filter(|v| v.fields.is_empty()).collect();

    let from_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let db_value = extract_custom_value(&v.attrs)
            .unwrap_or_else(|| convert_to_snake_case(&variant_name.to_string()));

        quote! {
            #name::#variant_name => #db_value,
        }
    });

    let display_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let db_value = extract_custom_value(&v.attrs)
            .unwrap_or_else(|| convert_to_snake_case(&variant_name.to_string()));

        quote! {
            #name::#variant_name => write!(f, #db_value),
        }
    });

    let decode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let db_value = extract_custom_value(&v.attrs)
            .unwrap_or_else(|| convert_to_snake_case(&variant_name.to_string()));

        quote! {
            #db_value => Ok(#name::#variant_name),
        }
    });

    let encode_patterns = unit_variants.iter().map(|v| {
        let variant_name = &v.ident;
        let db_value = extract_custom_value(&v.attrs)
            .unwrap_or_else(|| convert_to_snake_case(&variant_name.to_string()));

        quote! {
            #name::#variant_name => #db_value,
        }
    });

    let expanded = quote! {
        impl #name {
            /// Get the PostgreSQL enum type name
            pub const fn postgres_type_name() -> &'static str {
                #postgres_type
            }

            /// Convert to database value string
            pub fn to_db_value(&self) -> &'static str {
                match self {
                    #(#encode_patterns)*
                    _ => "unknown",
                }
            }

            /// Parse from database value string
            pub fn from_db_value(s: &str) -> Result<Self, String> {
                match s {
                    #(#decode_patterns)*
                    _ => Err(format!("Unknown {} value: {}", stringify!(#name), s)),
                }
            }
        }

        impl From<#name> for sea_query::Value {
            fn from(value: #name) -> Self {
                let s = value.to_db_value();
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
                sqlx::postgres::PgTypeInfo::with_name(#postgres_type)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for #name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>
            ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                Self::from_db_value(s).map_err(|e| e.into())
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for #name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer
            ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_db_value();
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

/// Extract custom postgres type from deen attribute
fn extract_postgres_type(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("deen") {
            if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
                let tokens = tokens.clone();
                if let Ok(MetaListParser(metas)) = parse2::<MetaListParser>(tokens) {
                    for meta in metas {
                        if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
                            if path.is_ident("postgres_type") {
                                if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract custom db value from deen attribute
fn extract_custom_value(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("deen") {
            if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
                let tokens = tokens.clone();
                if let Ok(MetaListParser(metas)) = parse2::<MetaListParser>(tokens) {
                    for meta in metas {
                        if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
                            if path.is_ident("value") {
                                if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Convert Rust enum name to PostgreSQL enum type name
fn convert_to_postgres_enum_type(name: &str) -> String {
    format!("{}_enum", convert_to_snake_case(name))
}

/// Convert CamelCase to snake_case
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
