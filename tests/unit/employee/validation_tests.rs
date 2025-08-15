use api::v1::employee::models::CreateEmployee;

#[cfg(test)]
mod employee_validation_tests {
    use super::*;

    // Helper function to validate employee name
    fn validate_employee_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Employee name cannot be empty".to_string());
        }
        
        if name.len() < 2 {
            return Err("Employee name must be at least 2 characters long".to_string());
        }
        
        if name.len() > 100 {
            return Err("Employee name must not exceed 100 characters".to_string());
        }
        
        // Check for only alphabetic characters and spaces
        if !name.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '\'') {
            return Err("Employee name can only contain letters, spaces, hyphens, and apostrophes".to_string());
        }
        
        // Check for multiple consecutive spaces
        if name.contains("  ") {
            return Err("Employee name cannot contain multiple consecutive spaces".to_string());
        }
        
        // Check for leading/trailing spaces
        if name.trim() != name {
            return Err("Employee name cannot have leading or trailing spaces".to_string());
        }
        
        Ok(())
    }

    // Helper function to validate employee role
    fn validate_employee_role(role: &str) -> Result<(), String> {
        if role.is_empty() {
            return Err("Employee role cannot be empty".to_string());
        }
        
        if role.len() < 2 {
            return Err("Employee role must be at least 2 characters long".to_string());
        }
        
        if role.len() > 50 {
            return Err("Employee role must not exceed 50 characters".to_string());
        }
        
        // Check for valid characters
        if !role.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-') {
            return Err("Employee role can only contain letters, numbers, spaces, and hyphens".to_string());
        }
        
        Ok(())
    }

    // Helper function to validate email format
    fn validate_email(email: &str) -> Result<(), String> {
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        
        if email.len() > 254 {
            return Err("Email must not exceed 254 characters".to_string());
        }
        
        // Check for consecutive dots
        if email.contains("..") {
            return Err("Email cannot contain consecutive dots".to_string());
        }
        
        // Check for valid domain
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid email format".to_string());
        }
        
        let local_part = parts[0];
        let domain = parts[1];
        
        // Local part validation
        if local_part.is_empty() {
            return Err("Invalid email format".to_string());
        }
        
        if local_part.len() > 64 {
            return Err("Invalid email format".to_string());
        }
        
        // Domain validation
        if domain.is_empty() {
            return Err("Invalid email format".to_string());
        }
        
        if domain.starts_with('.') || domain.ends_with('.') {
            return Err("Domain cannot start or end with a dot".to_string());
        }
        
        if domain.contains("..") {
            return Err("Domain cannot contain consecutive dots".to_string());
        }
        
        // Check domain parts
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.len() < 1 {
            return Err("Invalid email format".to_string());
        }
        
        for part in &domain_parts {
            if part.is_empty() {
                return Err("Invalid email format".to_string());
            }
            if part.len() > 63 {
                return Err("Invalid email format".to_string());
            }
        }
        
        // Check TLD - allow single-part domains like localhost
        if domain_parts.len() > 1 {
            let tld = domain_parts.last().unwrap();
            if tld.len() < 2 || !tld.chars().all(|c| c.is_alphabetic()) {
                return Err("TLD must be at least 2 alphabetic characters".to_string());
            }
        }
        
        // Basic character validation for local part
        let valid_local_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.!#$%&'*+/=?^_`{|}~-";
        if !local_part.chars().all(|c| valid_local_chars.contains(c)) {
            return Err("Invalid email format".to_string());
        }
        
        // Basic character validation for domain
        if !domain.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
            return Err("Invalid email format".to_string());
        }
        
        Ok(())
    }

    #[test]
    fn test_valid_employee_names() {
        let valid_names = vec![
            "John Doe",
            "Alice",
            "Bob Smith-Jones",
            "O'Connor",
            "Jean-Luc Picard",
            "María José García-López",
        ];
        
        for name in valid_names {
            assert!(validate_employee_name(name).is_ok(), "Name '{}' should be valid", name);
        }
    }

    #[test]
    fn test_invalid_employee_name_empty() {
        let result = validate_employee_name("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee name cannot be empty");
    }

    #[test]
    fn test_invalid_employee_name_too_short() {
        let result = validate_employee_name("A");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee name must be at least 2 characters long");
    }

    #[test]
    fn test_invalid_employee_name_too_long() {
        let long_name = "a".repeat(101);
        let result = validate_employee_name(&long_name);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee name must not exceed 100 characters");
    }

    #[test]
    fn test_invalid_employee_name_special_chars() {
        let result = validate_employee_name("John@Doe");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("can only contain letters, spaces, hyphens, and apostrophes"));
    }

    #[test]
    fn test_invalid_employee_name_consecutive_spaces() {
        let result = validate_employee_name("John  Doe");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee name cannot contain multiple consecutive spaces");
    }

    #[test]
    fn test_invalid_employee_name_leading_space() {
        let result = validate_employee_name(" John Doe");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee name cannot have leading or trailing spaces");
    }

    #[test]
    fn test_valid_employee_roles() {
        let valid_roles = vec![
            "Software Engineer",
            "Manager",
            "Sales Representative",
            "HR-Director",
            "Team Lead",
            "Senior Developer",
        ];
        
        for role in valid_roles {
            assert!(validate_employee_role(role).is_ok(), "Role '{}' should be valid", role);
        }
    }

    #[test]
    fn test_invalid_employee_role_empty() {
        let result = validate_employee_role("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee role cannot be empty");
    }

    #[test]
    fn test_invalid_employee_role_too_short() {
        let result = validate_employee_role("A");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee role must be at least 2 characters long");
    }

    #[test]
    fn test_invalid_employee_role_too_long() {
        let long_role = "a".repeat(51);
        let result = validate_employee_role(&long_role);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Employee role must not exceed 50 characters");
    }

    #[test]
    fn test_invalid_employee_role_special_chars() {
        let result = validate_employee_role("Engineer@Level");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("can only contain letters, numbers, spaces, and hyphens"));
    }

    #[test]
    fn test_valid_emails() {
        let valid_emails = vec![
            "user@example.com",
            "test.email@domain.org",
            "firstname.lastname@company.co.uk",
            "user+tag@example.com",
            "admin@localhost",
            "123@test.com",
        ];
        
        for email in valid_emails {
            assert!(validate_email(email).is_ok(), "Email '{}' should be valid", email);
        }
    }

    #[test]
    fn test_invalid_email_empty() {
        let result = validate_email("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email cannot be empty");
    }

    #[test]
    fn test_invalid_email_no_at_symbol() {
        let result = validate_email("invalid.email.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email format"));
    }

    #[test]
    fn test_invalid_email_multiple_at_symbols() {
        let result = validate_email("user@@example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email format"));
    }

    #[test]
    fn test_invalid_email_consecutive_dots() {
        let result = validate_email("user@example..com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email cannot contain consecutive dots");
    }

    #[test]
    fn test_invalid_email_domain_starts_with_dot() {
        let result = validate_email("user@.example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Domain cannot start or end with a dot");
    }

    #[test]
    fn test_invalid_email_tld_too_short() {
        let result = validate_email("user@example.c");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "TLD must be at least 2 alphabetic characters");
    }

    #[test]
    fn test_invalid_email_tld_numeric() {
        let result = validate_email("user@example.123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "TLD must be at least 2 alphabetic characters");
    }

    #[test]
    fn test_valid_create_employee() {
        let employee = CreateEmployee {
            name: "John Doe".to_string(),
            role: "Software Engineer".to_string(),
            email: "john.doe@example.com".to_string(),
        };
        
        assert!(validate_employee_name(&employee.name).is_ok());
        assert!(validate_employee_role(&employee.role).is_ok());
        assert!(validate_email(&employee.email).is_ok());
    }

    #[test]
    fn test_create_employee_with_invalid_name() {
        let employee = CreateEmployee {
            name: "".to_string(),
            role: "Software Engineer".to_string(),
            email: "john.doe@example.com".to_string(),
        };
        
        assert!(validate_employee_name(&employee.name).is_err());
        assert!(validate_employee_role(&employee.role).is_ok());
        assert!(validate_email(&employee.email).is_ok());
    }

    #[test]
    fn test_create_employee_with_invalid_role() {
        let employee = CreateEmployee {
            name: "John Doe".to_string(),
            role: "".to_string(),
            email: "john.doe@example.com".to_string(),
        };
        
        assert!(validate_employee_name(&employee.name).is_ok());
        assert!(validate_employee_role(&employee.role).is_err());
        assert!(validate_email(&employee.email).is_ok());
    }

    #[test]
    fn test_create_employee_with_invalid_email() {
        let employee = CreateEmployee {
            name: "John Doe".to_string(),
            role: "Software Engineer".to_string(),
            email: "invalid-email".to_string(),
        };
        
        assert!(validate_employee_name(&employee.name).is_ok());
        assert!(validate_employee_role(&employee.role).is_ok());
        assert!(validate_email(&employee.email).is_err());
    }

    #[test]
    fn test_create_employee_with_all_invalid_fields() {
        let employee = CreateEmployee {
            name: "".to_string(),
            role: "".to_string(),
            email: "invalid".to_string(),
        };
        
        assert!(validate_employee_name(&employee.name).is_err());
        assert!(validate_employee_role(&employee.role).is_err());
        assert!(validate_email(&employee.email).is_err());
    }
}