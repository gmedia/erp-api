use api::v1::auth::models::{LoginRequest, RegisterRequest};

#[cfg(test)]
mod password_validation_tests {
    use super::*;

    // Helper function to validate password strength
    fn validate_password(password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        
        if password.len() > 128 {
            return Err("Password must not exceed 128 characters".to_string());
        }
        
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if !has_uppercase {
            return Err("Password must contain at least one uppercase letter".to_string());
        }
        
        if !has_lowercase {
            return Err("Password must contain at least one lowercase letter".to_string());
        }
        
        if !has_digit {
            return Err("Password must contain at least one digit".to_string());
        }
        
        if !has_special {
            return Err("Password must contain at least one special character".to_string());
        }
        
        Ok(())
    }

    // Helper function to validate username
    fn validate_username(username: &str) -> Result<(), String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        if username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }
        
        if username.len() > 32 {
            return Err("Username must not exceed 32 characters".to_string());
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain letters, numbers, underscores, and hyphens".to_string());
        }
        
        if username.starts_with('-') || username.ends_with('-') {
            return Err("Username cannot start or end with a hyphen".to_string());
        }
        
        if username.starts_with('_') || username.ends_with('_') {
            return Err("Username cannot start or end with an underscore".to_string());
        }
        
        Ok(())
    }

    #[test]
    fn test_valid_password() {
        let valid_passwords = vec![
            "ValidPass123!",
            "MySecureP@ssw0rd",
            "Complex123$%^",
            "StrongP@ss1",
        ];
        
        for password in valid_passwords {
            assert!(validate_password(password).is_ok(), "Password '{}' should be valid", password);
        }
    }

    #[test]
    fn test_invalid_password_too_short() {
        let result = validate_password("Short1!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must be at least 8 characters long");
    }

    #[test]
    fn test_invalid_password_no_uppercase() {
        let result = validate_password("lowercase123!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must contain at least one uppercase letter");
    }

    #[test]
    fn test_invalid_password_no_lowercase() {
        let result = validate_password("UPPERCASE123!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must contain at least one lowercase letter");
    }

    #[test]
    fn test_invalid_password_no_digit() {
        let result = validate_password("NoDigitsHere!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must contain at least one digit");
    }

    #[test]
    fn test_invalid_password_no_special_char() {
        let result = validate_password("NoSpecialChar123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must contain at least one special character");
    }

    #[test]
    fn test_password_too_long() {
        let long_password = "a".repeat(129);
        let result = validate_password(&long_password);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must not exceed 128 characters");
    }

    #[test]
    fn test_valid_username() {
        let valid_usernames = vec![
            "user123",
            "test_user",
            "valid-name",
            "a1b2c3",
            "user-name_123",
        ];
        
        for username in valid_usernames {
            assert!(validate_username(username).is_ok(), "Username '{}' should be valid", username);
        }
    }

    #[test]
    fn test_invalid_username_empty() {
        let result = validate_username("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username cannot be empty");
    }

    #[test]
    fn test_invalid_username_too_short() {
        let result = validate_username("ab");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username must be at least 3 characters long");
    }

    #[test]
    fn test_invalid_username_too_long() {
        let long_username = "a".repeat(33);
        let result = validate_username(&long_username);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username must not exceed 32 characters");
    }

    #[test]
    fn test_invalid_username_special_chars() {
        let result = validate_username("user@name");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("can only contain letters, numbers, underscores, and hyphens"));
    }

    #[test]
    fn test_invalid_username_starts_with_hyphen() {
        let result = validate_username("-username");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username cannot start or end with a hyphen");
    }

    #[test]
    fn test_invalid_username_ends_with_underscore() {
        let result = validate_username("username_");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username cannot start or end with an underscore");
    }

    #[test]
    fn test_valid_login_request() {
        let login = LoginRequest {
            username: "validuser".to_string(),
            password: "ValidPass123!".to_string(),
        };
        
        assert!(validate_username(&login.username).is_ok());
        assert!(validate_password(&login.password).is_ok());
    }

    #[test]
    fn test_valid_register_request() {
        let register = RegisterRequest {
            username: "newuser".to_string(),
            password: "NewPass123!".to_string(),
        };
        
        assert!(validate_username(&register.username).is_ok());
        assert!(validate_password(&register.password).is_ok());
    }

    #[test]
    fn test_register_request_with_invalid_username() {
        let register = RegisterRequest {
            username: "ab".to_string(),
            password: "ValidPass123!".to_string(),
        };
        
        assert!(validate_username(&register.username).is_err());
        assert!(validate_password(&register.password).is_ok());
    }

    #[test]
    fn test_register_request_with_invalid_password() {
        let register = RegisterRequest {
            username: "validuser".to_string(),
            password: "weak".to_string(),
        };
        
        assert!(validate_username(&register.username).is_ok());
        assert!(validate_password(&register.password).is_err());
    }
}