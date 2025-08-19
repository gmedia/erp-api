# Page Testing Helper

This module provides comprehensive testing utilities for page routes in the ERP API.

## Overview

The `helper.rs` file contains all the necessary functions and structs to test page routes with:
- Actix-web server initialization for page routes
- Session management setup
- Inertia.js integration testing
- Form submission testing utilities
- Flash message testing

## Quick Start

```rust
use crate::page::helper::{setup_test_page_app_no_data, PageTestClient};

#[actix_web::test]
async fn test_page_loads() {
    let (_db_pool, _meili_client, server_url, _server_handle, _temp_dir) = 
        setup_test_page_app_no_data().await;

    let client = PageTestClient::new(server_url);
    let response = client.get_page("/").await.unwrap();
    
    assert!(response.contains("<!DOCTYPE html>"));
}
```

## Available Functions

### Server Setup Functions

- `setup_test_page_app()` - Full setup with configurable parameters
- `setup_test_page_app_no_data()` - Setup with clean database
- `setup_test_page_app_no_state()` - Minimal setup without database

### Testing Clients

- `PageTestClient` - Basic HTTP client with cookie support
- `FormTester` - Specialized for form testing
- `FlashMessageTester` - For testing flash messages
- `InertiaTester` - For testing Inertia.js responses

### Utility Functions

- `get_auth_token()` - Get JWT token for authenticated testing
- `create_test_user()` - Create test users
- `extract_csrf_token()` - Extract CSRF tokens from HTML
- `test_redirect()` - Test redirect responses

## Usage Examples

### Testing Page Loads
```rust
let client = PageTestClient::new(server_url);
let html = client.get_page("/contact").await.unwrap();
assert!(html.contains("Contact Us"));
```

### Testing Form Submissions
```rust
let form_tester = FormTester::new(server_url);
let data = json!({"title": "Test", "description": "Test desc"});
let response = form_tester.test_successful_submission("/todo/store", &data).await.unwrap();
assert!(response.status().is_success());
```

### Testing Flash Messages
```rust
let flash_tester = FlashMessageTester::new(server_url);
let html = client.get_page("/").await.unwrap();
assert!(flash_tester.has_success_message(&html).await);
```

### Testing Inertia Responses
```rust
let inertia_tester = InertiaTester::new(server_url);
let response = inertia_tester.test_inertia_response("/todo", "Todo/Index").await.unwrap();
assert_eq!(response["component"], "Todo/Index");
```

## Session Management

The helper automatically creates temporary directories for session storage and cleans them up after tests complete using `TempDir`.

## Database Testing

All setup functions return a database connection that can be used to:
- Seed test data
- Verify database changes
- Clean up after tests

## Best Practices

1. Use `#[serial]` attribute for tests that modify shared resources
2. Always clean up test data in teardown
3. Use the provided temporary directories for session storage
4. Test both success and error cases for forms
5. Verify both HTML responses and JSON API responses