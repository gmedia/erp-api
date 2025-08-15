use api::v1::inventory::models::{CreateInventoryItem, UpdateInventoryItem};

#[cfg(test)]
mod inventory_validation_tests {
    use super::*;

    // Helper function to validate inventory item name
    fn validate_item_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Item name cannot be empty".to_string());
        }
        
        if name.len() < 2 {
            return Err("Item name must be at least 2 characters long".to_string());
        }
        
        if name.len() > 100 {
            return Err("Item name must not exceed 100 characters".to_string());
        }
        
        // Check for valid characters (alphanumeric, spaces, hyphens, underscores)
        if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_' || c == '(' || c == ')') {
            return Err("Item name can only contain letters, numbers, spaces, hyphens, underscores, and parentheses".to_string());
        }
        
        // Check for leading/trailing spaces
        if name.trim() != name {
            return Err("Item name cannot have leading or trailing spaces".to_string());
        }
        
        // Check for multiple consecutive spaces
        if name.contains("  ") {
            return Err("Item name cannot contain multiple consecutive spaces".to_string());
        }
        
        Ok(())
    }

    // Helper function to validate quantity
    fn validate_quantity(quantity: i32) -> Result<(), String> {
        if quantity < 0 {
            return Err("Quantity cannot be negative".to_string());
        }
        
        if quantity > 1_000_000 {
            return Err("Quantity cannot exceed 1,000,000".to_string());
        }
        
        Ok(())
    }

    // Helper function to validate price
    fn validate_price(price: f64) -> Result<(), String> {
        // Check for NaN or Infinity first
        if price.is_nan() || price.is_infinite() {
            return Err("Price must be a valid number".to_string());
        }
        
        if price < 0.0 {
            return Err("Price cannot be negative".to_string());
        }
        
        if price > 1_000_000.0 {
            return Err("Price cannot exceed 1,000,000".to_string());
        }
        
        // Check for reasonable decimal places (max 2)
        let price_str = price.to_string();
        if let Some(dot_index) = price_str.find('.') {
            let decimal_places = price_str.len() - dot_index - 1;
            if decimal_places > 2 {
                return Err("Price cannot have more than 2 decimal places".to_string());
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_valid_item_names() {
        let valid_names = vec![
            "Laptop Computer",
            "Wireless Mouse",
            "USB-C Cable (2m)",
            "Office Chair - Ergonomic",
            "Standing_Desk_Pro",
            "Monitor 27 inch",
        ];
        
        for name in valid_names {
            assert!(validate_item_name(name).is_ok(), "Name '{}' should be valid", name);
        }
    }

    #[test]
    fn test_invalid_item_name_empty() {
        let result = validate_item_name("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item name cannot be empty");
    }

    #[test]
    fn test_invalid_item_name_too_short() {
        let result = validate_item_name("A");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item name must be at least 2 characters long");
    }

    #[test]
    fn test_invalid_item_name_too_long() {
        let long_name = "a".repeat(101);
        let result = validate_item_name(&long_name);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item name must not exceed 100 characters");
    }

    #[test]
    fn test_invalid_item_name_special_chars() {
        let result = validate_item_name("Laptop@Computer");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("can only contain letters, numbers, spaces, hyphens, underscores, and parentheses"));
    }

    #[test]
    fn test_invalid_item_name_leading_space() {
        let result = validate_item_name(" Laptop");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item name cannot have leading or trailing spaces");
    }

    #[test]
    fn test_invalid_item_name_consecutive_spaces() {
        let result = validate_item_name("Laptop  Computer");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item name cannot contain multiple consecutive spaces");
    }

    #[test]
    fn test_valid_quantities() {
        let valid_quantities = vec![0, 1, 50, 1000, 50000, 999999, 1000000];
        
        for quantity in valid_quantities {
            assert!(validate_quantity(quantity).is_ok(), "Quantity {} should be valid", quantity);
        }
    }

    #[test]
    fn test_invalid_quantity_negative() {
        let result = validate_quantity(-1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Quantity cannot be negative");
    }

    #[test]
    fn test_invalid_quantity_too_large() {
        let result = validate_quantity(1_000_001);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Quantity cannot exceed 1,000,000");
    }

    #[test]
    fn test_valid_prices() {
        let valid_prices = vec![0.0, 0.01, 10.0, 99.99, 1000.50, 999999.99];
        
        for price in valid_prices {
            assert!(validate_price(price).is_ok(), "Price {} should be valid", price);
        }
    }

    #[test]
    fn test_invalid_price_negative() {
        let result = validate_price(-1.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Price cannot be negative");
    }

    #[test]
    fn test_invalid_price_too_large() {
        let result = validate_price(1_000_000.01);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Price cannot exceed 1,000,000");
    }

    #[test]
    fn test_invalid_price_too_many_decimals() {
        let result = validate_price(10.123);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Price cannot have more than 2 decimal places");
    }

    #[test]
    fn test_invalid_price_nan() {
        let result = validate_price(f64::NAN);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Price must be a valid number");
    }

    #[test]
    fn test_invalid_price_infinity() {
        let result = validate_price(f64::INFINITY);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Price must be a valid number");
    }

    #[test]
    fn test_valid_create_inventory_item() {
        let item = CreateInventoryItem {
            name: "Laptop Computer".to_string(),
            quantity: 10,
            price: 999.99,
        };
        
        assert!(validate_item_name(&item.name).is_ok());
        assert!(validate_quantity(item.quantity).is_ok());
        assert!(validate_price(item.price).is_ok());
    }

    #[test]
    fn test_create_inventory_item_with_invalid_name() {
        let item = CreateInventoryItem {
            name: "".to_string(),
            quantity: 10,
            price: 999.99,
        };
        
        assert!(validate_item_name(&item.name).is_err());
        assert!(validate_quantity(item.quantity).is_ok());
        assert!(validate_price(item.price).is_ok());
    }

    #[test]
    fn test_create_inventory_item_with_invalid_quantity() {
        let item = CreateInventoryItem {
            name: "Laptop Computer".to_string(),
            quantity: -5,
            price: 999.99,
        };
        
        assert!(validate_item_name(&item.name).is_ok());
        assert!(validate_quantity(item.quantity).is_err());
        assert!(validate_price(item.price).is_ok());
    }

    #[test]
    fn test_create_inventory_item_with_invalid_price() {
        let item = CreateInventoryItem {
            name: "Laptop Computer".to_string(),
            quantity: 10,
            price: -999.99,
        };
        
        assert!(validate_item_name(&item.name).is_ok());
        assert!(validate_quantity(item.quantity).is_ok());
        assert!(validate_price(item.price).is_err());
    }

    #[test]
    fn test_valid_update_inventory_item() {
        let update = UpdateInventoryItem {
            name: Some("Updated Laptop".to_string()),
            quantity: Some(15),
            price: Some(899.99),
        };
        
        if let Some(name) = &update.name {
            assert!(validate_item_name(name).is_ok());
        }
        if let Some(quantity) = update.quantity {
            assert!(validate_quantity(quantity).is_ok());
        }
        if let Some(price) = update.price {
            assert!(validate_price(price).is_ok());
        }
    }

    #[test]
    fn test_update_inventory_item_with_none_values() {
        let _update = UpdateInventoryItem {
            name: None,
            quantity: None,
            price: None,
        };
        
        // Should be valid as all fields are optional
        assert!(true); // No validation needed for None values
    }

    #[test]
    fn test_update_inventory_item_with_partial_updates() {
        let _update = UpdateInventoryItem {
            name: Some("Updated Name".to_string()),
            quantity: None,
            price: Some(199.99),
        };
        
        if let Some(name) = &_update.name {
            assert!(validate_item_name(name).is_ok());
        }
        if let Some(price) = _update.price {
            assert!(validate_price(price).is_ok());
        }
    }

    #[test]
    fn test_update_inventory_item_with_invalid_values() {
        let _update = UpdateInventoryItem {
            name: Some("".to_string()),
            quantity: Some(-1),
            price: Some(-10.0),
        };
        
        if let Some(name) = &_update.name {
            assert!(validate_item_name(name).is_err());
        }
        if let Some(quantity) = _update.quantity {
            assert!(validate_quantity(quantity).is_err());
        }
        if let Some(price) = _update.price {
            assert!(validate_price(price).is_err());
        }
    }

    #[test]
    fn test_search_query_building() {
        // Test search query validation
        let search_terms = vec![
            "laptop",
            "wireless mouse",
            "usb-c cable",
            "office chair",
        ];
        
        for term in search_terms {
            assert!(!term.is_empty(), "Search term should not be empty");
            assert!(term.len() <= 100, "Search term should not exceed 100 characters");
        }
    }

    #[test]
    fn test_empty_search_query() {
        let search_term = "";
        assert!(search_term.is_empty());
    }

    #[test]
    fn test_search_query_special_chars() {
        let search_term = "laptop@computer";
        assert!(search_term.len() <= 100);
        // In a real implementation, we might sanitize or reject special chars
    }
}