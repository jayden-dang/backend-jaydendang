/// Implement `From<FromType>` for `ToType`
#[macro_export]
macro_rules! impl_from_domain {
    (
        from: $from_ty:ty,
        to: $to_ty:ty,
        fields: { $( $to_field:ident <- $from_field:ident ),* $(,)? }
    ) => {
        impl From<$from_ty> for $to_ty {
            fn from(src: $from_ty) -> Self {
                Self {
                    $( $to_field: src.$from_field ),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_into_record {
    (
        from: $from_ty:ty,
        to: $to_ty:ty,
        fields: { $( $to_field:ident <- $from_field:ident ),* $(,)? }
    ) => {
        impl From<$from_ty> for $to_ty {
            fn from(src: $from_ty) -> Self {
                Self {
                    $( $to_field: src.$from_field ),*
                }
            }
        }
    };
}
