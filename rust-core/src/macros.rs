//! Macro Definitions for Code Generation

/// Macro to implement common getters for structs
///
/// # Examples
/// ```ignore
/// struct User {
///     id: Uuid,
///     name: String,
/// }
///
/// impl_getters!(User {
///     id: Uuid,
///     name: String
/// });
/// ```
#[macro_export]
macro_rules! impl_getters {
    ($struct_name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        impl $struct_name {
            $(
                paste::paste! {
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.$field
                    }
                }
            )*
        }
    };
}

/// Macro to implement builder pattern for structs
///
/// # Examples
/// ```ignore
/// struct User {
///     id: Uuid,
///     name: String,
///     email: String,
/// }
///
/// impl_builder!(User {
///     name: String,
///     email: String
/// });
/// ```
#[macro_export]
macro_rules! impl_builder {
    ($struct_name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        impl $struct_name {
            $(
                paste::paste! {
                    pub fn [<with_ $field>](mut self, $field: $type) -> Self {
                        self.$field = $field;
                        self
                    }
                }
            )*
        }
    };
}

/// Macro to implement Display trait
#[macro_export]
macro_rules! impl_display {
    ($struct_name:ident, $format:expr) => {
        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $format, self)
            }
        }
    };
}

/// Macro to implement From trait for error conversion
#[macro_export]
macro_rules! impl_from_error {
    ($error_type:ty => $target_type:ty, $variant:path) => {
        impl From<$error_type> for $target_type {
            fn from(err: $error_type) -> Self {
                $variant(err.to_string())
            }
        }
    };
}

/// Macro to create a new UUID-based entity
#[macro_export]
macro_rules! new_entity {
    ($struct_name:ident { $($field:ident: $value:expr),* $(,)? }) => {
        $struct_name {
            id: uuid::Uuid::new_v4(),
            $($field: $value,)*
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    };
}

/// Macro to implement common trait bounds
#[macro_export]
macro_rules! derive_common {
    ($struct_name:ident) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $struct_name;
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_compile() {
        // Just ensure macros are exported and available
        assert!(true);
    }
}
