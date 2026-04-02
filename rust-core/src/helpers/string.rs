//! String Helper Functions

use rand::Rng;

/// Convert string to slug format (lowercase with hyphens)
///
/// # Examples
/// ```
/// use rust_core::str_slug;
///
/// assert_eq!(str_slug("Hello World"), "hello-world");
/// assert_eq!(str_slug("Product Name 123"), "product-name-123");
/// ```
pub fn str_slug(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert string to snake_case
///
/// # Examples
/// ```
/// use rust_core::str_snake;
///
/// assert_eq!(str_snake("HelloWorld"), "hello_world");
/// assert_eq!(str_snake("ProductName"), "product_name");
/// ```
pub fn str_snake(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

/// Convert string to camelCase
///
/// # Examples
/// ```
/// use rust_core::str_camel;
///
/// assert_eq!(str_camel("hello_world"), "helloWorld");
/// assert_eq!(str_camel("product-name"), "productName");
/// ```
pub fn str_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' || c == '-' || c.is_whitespace() {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert string to PascalCase
///
/// # Examples
/// ```
/// use rust_core::str_pascal;
///
/// assert_eq!(str_pascal("hello_world"), "HelloWorld");
/// assert_eq!(str_pascal("product-name"), "ProductName");
/// ```
pub fn str_pascal(s: &str) -> String {
    let camel = str_camel(s);
    if camel.is_empty() {
        return camel;
    }

    let mut chars = camel.chars();
    let first = chars.next().unwrap().to_uppercase().next().unwrap();
    format!("{}{}", first, chars.as_str())
}

/// Generate random string with given length
///
/// # Examples
/// ```
/// use rust_core::str_random;
///
/// let random = str_random(10);
/// assert_eq!(random.len(), 10);
/// ```
pub fn str_random(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Truncate string to given length with suffix
///
/// # Examples
/// ```
/// use rust_core::str_limit;
///
/// assert_eq!(str_limit("Hello World", 5, "..."), "Hello...");
/// assert_eq!(str_limit("Short", 10, "..."), "Short");
/// ```
pub fn str_limit(s: &str, length: usize, suffix: &str) -> String {
    if s.len() <= length {
        s.to_string()
    } else {
        format!("{}{}", &s[..length], suffix)
    }
}

/// Check if string starts with any of the given needles
pub fn str_starts_with_any(s: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| s.starts_with(needle))
}

/// Check if string ends with any of the given needles
pub fn str_ends_with_any(s: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| s.ends_with(needle))
}

/// Check if string contains any of the given needles
pub fn str_contains_any(s: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| s.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_slug() {
        assert_eq!(str_slug("Hello World"), "hello-world");
        assert_eq!(str_slug("Product Name 123"), "product-name-123");
        assert_eq!(str_slug("Test___Value"), "test-value");
    }

    #[test]
    fn test_str_snake() {
        assert_eq!(str_snake("HelloWorld"), "hello_world");
        assert_eq!(str_snake("ProductName"), "product_name");
        assert_eq!(str_snake("XMLHttpRequest"), "x_m_l_http_request");
    }

    #[test]
    fn test_str_camel() {
        assert_eq!(str_camel("hello_world"), "helloWorld");
        assert_eq!(str_camel("product-name"), "productName");
        assert_eq!(str_camel("test value"), "testValue");
    }

    #[test]
    fn test_str_pascal() {
        assert_eq!(str_pascal("hello_world"), "HelloWorld");
        assert_eq!(str_pascal("product-name"), "ProductName");
    }

    #[test]
    fn test_str_random() {
        let random = str_random(10);
        assert_eq!(random.len(), 10);
        assert!(random.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_str_limit() {
        assert_eq!(str_limit("Hello World", 5, "..."), "Hello...");
        assert_eq!(str_limit("Short", 10, "..."), "Short");
    }

    #[test]
    fn test_str_starts_with_any() {
        assert!(str_starts_with_any("hello world", &["hello", "hi"]));
        assert!(!str_starts_with_any("hello world", &["world", "hi"]));
    }

    #[test]
    fn test_str_ends_with_any() {
        assert!(str_ends_with_any("hello world", &["world", "test"]));
        assert!(!str_ends_with_any("hello world", &["hello", "test"]));
    }

    #[test]
    fn test_str_contains_any() {
        assert!(str_contains_any("hello world", &["lo wo", "test"]));
        assert!(!str_contains_any("hello world", &["xyz", "abc"]));
    }
}
