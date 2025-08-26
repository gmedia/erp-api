use crate::error::ApiError;

/// Generic validation trait
pub trait Validatable {
    fn validate(&self) -> Result<(), ApiError>;
}

/// Common validation utilities
pub mod validation {
    use crate::error::ApiError;

    pub fn validate_non_negative(value: i32, field: &str) -> Result<(), ApiError> {
        if value < 0 {
            Err(ApiError::ValidationError(format!(
                "{} cannot be negative",
                field
            )))
        } else {
            Ok(())
        }
    }

    pub fn validate_non_negative_float(value: f64, field: &str) -> Result<(), ApiError> {
        if value < 0.0 {
            Err(ApiError::ValidationError(format!(
                "{} cannot be negative",
                field
            )))
        } else {
            Ok(())
        }
    }

    pub fn validate_email(email: &str) -> Result<(), ApiError> {
        if !email.contains('@') {
            Err(ApiError::ValidationError(
                "Invalid email format".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Common utilities for entity operations
pub mod entity_utils {
    use uuid::Uuid;

    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }
}