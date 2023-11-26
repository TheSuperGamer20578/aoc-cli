#[macro_export]
macro_rules! value_enum {
    {
        $(#[$attr:meta])*
        $vis:vis enum $name:ident: $prop_type:ty {
            $($variant:ident = $value:expr),*$(,)?
        }
    } => {
        $(#[$attr])*
        $vis enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn value(&self) -> $prop_type {
                match self {
                    $(Self::$variant => $value,)*
                }
            }
        }

        impl From<$name> for $prop_type {
            fn from(value: $name) -> Self {
                value.value()
            }
        }
    };
}
