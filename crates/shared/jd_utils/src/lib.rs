pub mod config;
mod error;
pub mod macros;
pub mod regex;
pub mod time;

pub use macros::*;

pub type Result<T> = std::result::Result<T, error::Error>;

#[macro_export]
macro_rules! impl_sqlx_encode_decode_enum {
    ($enum_type:ty, { $($variant:ident),* $(,)? }) => {
        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $enum_type {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>
            ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                match s {
                    $(
                        _ if s == stringify!($variant).to_lowercase() => Ok(<$enum_type>::$variant),
                    )*
                    _ => Err(format!("Unknown {}: {}", stringify!($enum_type), s).into()),
                }
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $enum_type {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer
            ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = match self {
                    $(
                        <$enum_type>::$variant => stringify!($variant).to_lowercase(),
                    )*
                    _ => return Err("Complex enum variants are not supported for DB storage".into()),
                };
                let s_ref: &str = &s;
                <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s_ref, buf)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_value_from_enum {
    ($enum_type:ty, { $($variant:ident),* $(,)? }) => {
        impl From<$enum_type> for sea_query::Value {
            fn from(value: $enum_type) -> Self {
                let s = match value {
                    $(<$enum_type>::$variant => stringify!($variant).to_lowercase(),)*
                    _ => return sea_query::Value::String(None),
                };
                sea_query::Value::String(Some(Box::new(s)))
            }
        }
    };
}
