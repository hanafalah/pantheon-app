//! Validation Utilities
//!
//! Helper functions for input validation

use super::error::{AppError, ValidationErrorDetail};
use regex::Regex;
use uuid::Uuid;

/// Validator helper
pub struct Validator;

impl Validator {
    /// Validate email format
    pub fn validate_email(email: &str) -> Result<(), ValidationErrorDetail> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if email_regex.is_match(email) {
            Ok(())
        } else {
            Err(ValidationErrorDetail {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            })
        }
    }

    /// Validate required field
    pub fn validate_required(
        field_name: &str,
        value: &Option<String>,
    ) -> Result<(), ValidationErrorDetail> {
        if value.is_none() || value.as_ref().unwrap().is_empty() {
            Err(ValidationErrorDetail {
                field: field_name.to_string(),
                message: format!("{} is required", field_name),
            })
        } else {
            Ok(())
        }
    }

    /// Validate minimum length
    pub fn validate_min_length(
        field_name: &str,
        value: &str,
        min: usize,
    ) -> Result<(), ValidationErrorDetail> {
        if value.len() < min {
            Err(ValidationErrorDetail {
                field: field_name.to_string(),
                message: format!("{} must be at least {} characters", field_name, min),
            })
        } else {
            Ok(())
        }
    }

    /// Validate maximum length
    pub fn validate_max_length(
        field_name: &str,
        value: &str,
        max: usize,
    ) -> Result<(), ValidationErrorDetail> {
        if value.len() > max {
            Err(ValidationErrorDetail {
                field: field_name.to_string(),
                message: format!("{} must not exceed {} characters", field_name, max),
            })
        } else {
            Ok(())
        }
    }

    /// Validate UUID format
    pub fn validate_uuid(field_name: &str, value: &str) -> Result<Uuid, ValidationErrorDetail> {
        Uuid::parse_str(value).map_err(|_| ValidationErrorDetail {
            field: field_name.to_string(),
            message: format!("{} must be a valid UUID", field_name),
        })
    }

    /// Validate phone number (simple validation)
    pub fn validate_phone(phone: &str) -> Result<(), ValidationErrorDetail> {
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();

        if phone_regex.is_match(phone) {
            Ok(())
        } else {
            Err(ValidationErrorDetail {
                field: "phone".to_string(),
                message: "Invalid phone number format".to_string(),
            })
        }
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> Result<(), ValidationErrorDetail> {
        if password.len() < 8 {
            return Err(ValidationErrorDetail {
                field: "password".to_string(),
                message: "Password must be at least 8 characters".to_string(),
            });
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(ValidationErrorDetail {
                field: "password".to_string(),
                message: "Password must contain uppercase, lowercase, and digit".to_string(),
            });
        }

        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<(), ValidationErrorDetail> {
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();

        if url_regex.is_match(url) {
            Ok(())
        } else {
            Err(ValidationErrorDetail {
                field: "url".to_string(),
                message: "Invalid URL format".to_string(),
            })
        }
    }

    /// Validate numeric range
    pub fn validate_range(
        field_name: &str,
        value: i64,
        min: i64,
        max: i64,
    ) -> Result<(), ValidationErrorDetail> {
        if value < min || value > max {
            Err(ValidationErrorDetail {
                field: field_name.to_string(),
                message: format!("{} must be between {} and {}", field_name, min, max),
            })
        } else {
            Ok(())
        }
    }

    /// Collect validation errors and return AppError if any
    pub fn collect_errors(
        results: Vec<Result<(), ValidationErrorDetail>>,
    ) -> Result<(), AppError> {
        let errors: Vec<ValidationErrorDetail> =
            results.into_iter().filter_map(|r| r.err()).collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::ValidationError(errors))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(Validator::validate_email("test@example.com").is_ok());
        assert!(Validator::validate_email("invalid-email").is_err());
        assert!(Validator::validate_email("@example.com").is_err());
    }

    #[test]
    fn test_validate_required() {
        assert!(Validator::validate_required("name", &Some("value".to_string())).is_ok());
        assert!(Validator::validate_required("name", &None).is_err());
        assert!(Validator::validate_required("name", &Some("".to_string())).is_err());
    }

    #[test]
    fn test_validate_min_length() {
        assert!(Validator::validate_min_length("username", "testuser", 5).is_ok());
        assert!(Validator::validate_min_length("username", "test", 5).is_err());
    }

    #[test]
    fn test_validate_max_length() {
        assert!(Validator::validate_max_length("username", "test", 10).is_ok());
        assert!(Validator::validate_max_length("username", "verylongusername", 10).is_err());
    }

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Validator::validate_uuid("id", valid_uuid).is_ok());
        assert!(Validator::validate_uuid("id", "invalid-uuid").is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(Validator::validate_password("Password123").is_ok());
        assert!(Validator::validate_password("short").is_err());
        assert!(Validator::validate_password("nouppercase123").is_err());
        assert!(Validator::validate_password("NOLOWERCASE123").is_err());
        assert!(Validator::validate_password("NoDigits").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(Validator::validate_range("age", 25, 18, 100).is_ok());
        assert!(Validator::validate_range("age", 17, 18, 100).is_err());
        assert!(Validator::validate_range("age", 101, 18, 100).is_err());
    }

    #[test]
    fn test_collect_errors() {
        let results = vec![
            Validator::validate_email("valid@email.com"),
            Validator::validate_min_length("username", "test", 5),
        ];

        let result = Validator::collect_errors(results);
        assert!(result.is_err());

        match result {
            Err(AppError::ValidationError(errors)) => {
                assert_eq!(errors.len(), 1);
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
