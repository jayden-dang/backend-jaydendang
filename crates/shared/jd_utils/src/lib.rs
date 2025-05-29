pub mod config;
pub mod error;
pub mod macros;
pub mod regex;
pub mod time;

pub use macros::*;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn convert_variant_name(s: &str) -> String {
  let mut result = String::new();
  let chars = s.chars().peekable();

  for c in chars {
    if c.is_uppercase() && !result.is_empty() {
      result.push('_');
    }
    result.push(c.to_lowercase().next().unwrap());
  }

  result
}

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
                        _ if s == $crate::convert_variant_name(stringify!($variant)) => Ok(<$enum_type>::$variant),
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
                        <$enum_type>::$variant => $crate::convert_variant_name(stringify!($variant)),
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
                    $(<$enum_type>::$variant => $crate::convert_variant_name(stringify!($variant)),)*
                    _ => return sea_query::Value::String(None),
                };
                sea_query::Value::String(Some(Box::new(s)))
            }
        }
    };
}

// Macro để handle transaction tự động
#[macro_export]
macro_rules! with_transaction {
    ($dbx:expr, $block:block) => {
        {
            $dbx.begin_txn().await?;
            let result = async move $block .await;
            match result {
                Ok(val) => {
                    $dbx.commit_txn().await?;
                    Ok(val)
                }
                Err(e) => {
                    $dbx.rollback_txn().await.ok(); // Ignore rollback errors
                    Err(e)
                }
            }
        }
    };
}
