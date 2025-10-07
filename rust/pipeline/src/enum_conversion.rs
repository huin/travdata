#[macro_export(local_inner_macros)]
macro_rules! impl_enum_conversions {
    ($enum_type:ident, $variant_and_type:ident, $value_name:literal) => {
        impl<'a> TryFrom<&'a $enum_type> for &'a $variant_and_type {
            type Error = ::anyhow::Error;

            fn try_from(value: &'a $enum_type) -> ::std::result::Result<Self, Self::Error> {
                match value {
                    $enum_type::$variant_and_type(variant_value) => Ok(variant_value),
                    _ => Err(::anyhow::anyhow!(
                        "{} is not of type {}, got {:?}",
                        $value_name,
                        std::stringify!($variant_and_type),
                        value,
                    )),
                }
            }
        }

        impl std::convert::From<$variant_and_type> for $enum_type {
            fn from(value: $variant_and_type) -> $enum_type {
                $enum_type::$variant_and_type(value)
            }
        }
    };
}
