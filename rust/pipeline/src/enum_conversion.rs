/// Generates a [TryFrom] implementation from [crate::specs::Spec] to the contained type.
#[macro_export(local_inner_macros)]
macro_rules! impl_enum_conversions {
    ($enum_type:ident, $variant_and_type:ident, $value_name:literal) => {
        impl<'a> ::std::convert::TryFrom<&'a $enum_type> for &'a $variant_and_type {
            type Error = $crate::StringError;

            fn try_from(
                value: &'a $enum_type,
            ) -> std::result::Result<&'a $variant_and_type, Self::Error> {
                match value {
                    $enum_type::$variant_and_type(variant_value) => Ok(variant_value),
                    got => Err($crate::StringError(::std::format!(
                        "expected variant {}, got {:?}",
                        ::std::stringify!($variant_and_type),
                        got,
                    ))),
                }
            }
        }

        impl ::std::convert::From<$variant_and_type> for $enum_type {
            fn from(value: $variant_and_type) -> $enum_type {
                $enum_type::$variant_and_type(value)
            }
        }
    };
}
