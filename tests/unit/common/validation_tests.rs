#[cfg(test)]
mod common_validation_tests {
    use std::collections::HashSet;

    // Helper function to validate string length
    fn validate_string_length(
        value: &str,
        min_length: usize,
        max_length: usize,
        field_name: &str,
    ) -> Result<(), String> {
        if value.len() < min_length {
            return Err(format!(
                "{} must be at least {} characters long",
                field_name, min_length
            ));
        }
        
        if value.len() > max_length {
            return Err(format!(
                "{} must not exceed {} characters",
                field_name, max_length
            ));
        }
        
        Ok(())
    }

    // Helper function to validate alphanumeric characters
    fn validate_alphanumeric(
        value: &str,
        allow_spaces: bool,
        allow_hyphens: bool,
        allow_underscores: bool,
        field_name: &str,
    ) -> Result<(), String> {
        let mut valid_chars = HashSet::new();
        
        // Add alphanumeric characters
        for c in 'a'..='z' {
            valid_chars.insert(c);
            valid_chars.insert(c.to_ascii_uppercase());
        }
        for c in '0'..='9' {
            valid_chars.insert(c);
        }
        
        if allow_spaces {
            valid_chars.insert(' ');
        }
        
        if allow_hyphens {
            valid_chars.insert('-');
        }
        
        if allow_underscores {
            valid_chars.insert('_');
        }
        
        for c in value.chars() {
            if !valid_chars.contains(&c) {
                let mut allowed = "letters and numbers".to_string();
                if allow_spaces {
                    allowed.push_str(", spaces");
                }
                if allow_hyphens {
                    allowed.push_str(", hyphens");
                }
                if allow_underscores {
                    allowed.push_str(", underscores");
                }
                
                return Err(format!(
                    "{} can only contain {}",
                    field_name, allowed
                ));
            }
        }
        
        Ok(())
    }

    // Helper function to sanitize input
    fn sanitize_input(input: &str) -> String {
        let mut result = String::new();
        let mut prev_char = None;
        
        for c in input.chars() {
            // Skip control characters
            if c.is_control() {
                continue;
            }
            
            // Replace multiple spaces with single space
            if c == ' ' {
                if prev_char != Some(' ') {
                    result.push(' ');
                }
            } else {
                result.push(c);
            }
            
            prev_char = Some(c);
        }
        
        // Trim leading and trailing spaces
        result.trim().to_string()
    }

    // Helper function to validate UUID format
    fn validate_uuid_format(uuid: &str) -> Result<(), String> {
        if uuid.len() != 36 {
            return Err("UUID must be 36 characters long".to_string());
        }
        
        let parts: Vec<&str> = uuid.split('-').collect();
        if parts.len() != 5 {
            return Err("UUID must have 5 parts separated by hyphens".to_string());
        }
        
        let expected_lengths = [8, 4, 4, 4, 12];
        for (i, &expected_len) in expected_lengths.iter().enumerate() {
            if parts[i].len() != expected_len {
                return Err(format!(
                    "UUID part {} must be {} characters long",
                    i + 1,
                    expected_len
                ));
            }
        }
        
        // Check for valid hex characters
        if !uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            return Err("UUID can only contain hexadecimal characters and hyphens".to_string());
        }
        
        Ok(())
    }

    #[test]
    fn test_validate_string_length_valid() {
        assert!(validate_string_length("hello", 1, 10, "Name").is_ok());
        assert!(validate_string_length("test", 4, 4, "Code").is_ok());
        assert!(validate_string_length("", 0, 5, "Description").is_ok());
    }

    #[test]
    fn test_validate_string_length_too_short() {
        let result = validate_string_length("hi", 3, 10, "Username");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Username must be at least 3 characters long");
    }

    #[test]
    fn test_validate_string_length_too_long() {
        let result = validate_string_length("verylongstring", 1, 5, "Title");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Title must not exceed 5 characters");
    }

    #[test]
    fn test_validate_alphanumeric_strict() {
        assert!(validate_alphanumeric("abc123", false, false, false, "Code").is_ok());
        assert!(validate_alphanumeric("ABC456", false, false, false, "Code").is_ok());
    }

    #[test]
    fn test_validate_alphanumeric_with_spaces() {
        assert!(validate_alphanumeric("hello world", true, false, false, "Name").is_ok());
        assert!(validate_alphanumeric("hello  world", true, false, false, "Name").is_ok());
    }

    #[test]
    fn test_validate_alphanumeric_with_hyphens() {
        assert!(validate_alphanumeric("test-name", false, true, false, "Slug").is_ok());
        assert!(validate_alphanumeric("test@name", false, true, false, "Slug").is_err());
    }

    #[test]
    fn test_validate_alphanumeric_with_underscores() {
        assert!(validate_alphanumeric("user_name", false, false, true, "Username").is_ok());
        assert!(validate_alphanumeric("user-name", false, false, true, "Username").is_err());
    }

    #[test]
    fn test_validate_alphanumeric_all_allowed() {
        assert!(validate_alphanumeric("test_user-name 123", true, true, true, "Field").is_ok());
    }

    #[test]
    fn test_sanitize_input_basic() {
        assert_eq!(sanitize_input("  hello  world  "), "hello world");
        assert_eq!(sanitize_input("hello\tworld"), "helloworld");
        assert_eq!(sanitize_input("hello\nworld"), "helloworld");
        assert_eq!(sanitize_input("  multiple   spaces  "), "multiple spaces");
    }

    #[test]
    fn test_sanitize_input_no_changes() {
        assert_eq!(sanitize_input("hello world"), "hello world");
        assert_eq!(sanitize_input("test"), "test");
        assert_eq!(sanitize_input(""), "");
    }

    #[test]
    fn test_sanitize_input_control_chars() {
        assert_eq!(sanitize_input("hello\x00world"), "helloworld");
        assert_eq!(sanitize_input("test\x01\x02\x03"), "test");
    }

    #[test]
    fn test_validate_uuid_format_valid() {
        let valid_uuids = vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "123e4567-e89b-12d3-a456-426614174000",
            "00000000-0000-0000-0000-000000000000",
        ];
        
        for uuid in valid_uuids {
            assert!(validate_uuid_format(uuid).is_ok(), "UUID '{}' should be valid", uuid);
        }
    }

    #[test]
    fn test_validate_uuid_format_invalid() {
        let invalid_uuids = vec![
            "550e8400-e29b-41d4-a716", // Too short
            "550e8400-e29b-41d4-a716-446655440000-extra", // Too long
            "550e8400e29b41d4a716446655440000", // No hyphens
            "550e8400-e29b-41d4-a716-44665544000g", // Invalid character 'g'
            "550e8400-e29b-41d4-a716-44665544000", // One character short
            "550e8400-e29b-41d4-a716-4466554400000", // One character extra
        ];
        
        for uuid in invalid_uuids {
            assert!(validate_uuid_format(uuid).is_err(), "UUID '{}' should be invalid", uuid);
        }
    }

    #[test]
    fn test_validate_uuid_format_wrong_parts() {
        let result = validate_uuid_format("550e8400-e29b-41d4-a716-4466554-40000");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uuid_format_wrong_lengths() {
        let result = validate_uuid_format("550e840-e29b-41d4-a716-446655440000");
        assert!(result.is_err());
    }

    #[test]
    fn test_data_sanitization_pipeline() {
        let raw_input = "  Hello   World!  \n\tThis is a test...  ";
        let sanitized = sanitize_input(raw_input);
        
        assert_eq!(sanitized, "Hello World! This is a test...");
        
        // Test that sanitized input passes validation
        assert!(validate_string_length(&sanitized, 1, 100, "Description").is_ok());
    }

    #[test]
    fn test_combined_validation() {
        let long_string = "a".repeat(101);
        let test_cases = vec![
            ("hello", true),
            ("hello world", true),
            ("hello-world", true),
            ("hello_world", true),
            ("hello@world", false),
            ("hello#world", false),
            ("", false),
            (&long_string, false),
        ];
        
        for (input, should_pass) in test_cases {
            let sanitized = sanitize_input(input);
            let length_valid = validate_string_length(&sanitized, 1, 100, "Test").is_ok();
            let alpha_valid = validate_alphanumeric(&sanitized, true, true, true, "Test").is_ok();
            
            let is_valid = length_valid && alpha_valid;
            assert_eq!(is_valid, should_pass, "Input '{}' validation failed", input);
        }
    }

    #[test]
    fn test_edge_case_strings() {
        // Test with Unicode characters
        let unicode_input = "Café résumé naïve";
        let result = validate_alphanumeric(unicode_input, true, false, false, "Text");
        assert!(result.is_err()); // Should fail due to non-ASCII characters
        
        // Test with only spaces
        let spaces_only = "   ";
        let sanitized = sanitize_input(spaces_only);
        assert_eq!(sanitized, "");
        
        // Test with mixed case
        let mixed_case = "Test STRING with MiXeD CaSe";
        assert!(validate_alphanumeric(&mixed_case, true, false, false, "Text").is_ok());
    }

    // Helper function to validate numeric strings
    fn validate_numeric_string(value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("Number cannot be empty".to_string());
        }
        
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err("Number can only contain digits".to_string());
        }
        
        Ok(())
    }

    #[test]
    fn test_numeric_string_validation() {
        let numeric_strings = vec![
            "123",
            "00123",
            "999999",
            "0",
        ];
        
        for num_str in numeric_strings {
            assert!(validate_numeric_string(num_str).is_ok());
        }
        
        let invalid_numeric = vec![
            "12.34",
            "-123",
            "12a34",
            "12 34",
        ];
        
        for invalid in invalid_numeric {
            let result = validate_numeric_string(invalid);
            assert!(result.is_err(), "Expected '{}' to fail validation", invalid);
        }
    }

    #[test]
    fn test_empty_and_whitespace_handling() {
        let empty_cases = vec![
            "",
            " ",
            "  ",
            "\t",
            "\n",
            "\r\n",
        ];
        
        for empty in empty_cases {
            let sanitized = sanitize_input(empty);
            assert!(sanitized.is_empty() || sanitized == " ");
            
            let result = validate_string_length(&sanitized, 1, 10, "Field");
            if !sanitized.is_empty() && sanitized != " " {
                assert!(result.is_ok());
            }
        }
    }
}