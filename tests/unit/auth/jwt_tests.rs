use api::middlewares::jwt::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod jwt_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SECRET: &str = "test_secret_key_for_unit_tests";
    const ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS256;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        sub: String,
        exp: usize,
    }

    impl From<Claims> for TestClaims {
        fn from(claims: Claims) -> Self {
            TestClaims {
                sub: claims.sub,
                exp: claims.exp,
            }
        }
    }

    // Helper function to create a valid token
    fn create_valid_token(subject: &str, expiration_hours: i64) -> String {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + (expiration_hours * 3600);
        
        let claims = TestClaims {
            sub: subject.to_string(),
            exp: expiration as usize,
        };
        
        encode(
            &Header::new(ALGORITHM),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap()
    }

    #[test]
    fn test_valid_token_generation_and_validation() {
        let subject = "test_user";
        let token = create_valid_token(subject, 1); // 1 hour expiration
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, subject);
    }

    #[test]
    fn test_expired_token() {
        let subject = "test_user";
        let token = create_valid_token(subject, -1); // 1 hour in the past
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_err());
        let error = decoded.unwrap_err();
        assert!(error.to_string().contains("ExpiredSignature"));
    }

    #[test]
    fn test_invalid_signature() {
        let subject = "test_user";
        let token = create_valid_token(subject, 1);
        
        let wrong_secret = "wrong_secret_key";
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(wrong_secret.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_err());
        let error = decoded.unwrap_err();
        assert!(error.to_string().contains("InvalidSignature"));
    }

    #[test]
    fn test_malformed_token() {
        let malformed_token = "not.a.valid.jwt.token";
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            malformed_token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_err());
    }

    #[test]
    fn test_empty_subject() {
        let token = create_valid_token("", 1);
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, "");
    }

    #[test]
    fn test_long_subject() {
        let long_subject = "a".repeat(1000);
        let token = create_valid_token(&long_subject, 1);
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, long_subject);
    }

    #[test]
    fn test_special_characters_in_subject() {
        let special_subject = "user@domain.com";
        let token = create_valid_token(special_subject, 1);
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, special_subject);
    }

    #[test]
    fn test_different_algorithm() {
        let subject = "test_user";
        let claims = TestClaims {
            sub: subject.to_string(),
            exp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                + 3600) as usize,
        };
        
        // Create token with HS512 instead of HS256
        let token = encode(
            &Header::new(jsonwebtoken::Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap();
        
        // Try to validate with HS256
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_err());
    }

    #[test]
    fn test_claims_struct_conversion() {
        let original_claims = Claims {
            sub: "test_user".to_string(),
            exp: 1234567890,
        };
        
        let test_claims: TestClaims = original_claims.into();
        
        assert_eq!(test_claims.sub, "test_user");
        assert_eq!(test_claims.exp, 1234567890);
    }

    #[test]
    fn test_token_with_no_expiration() {
        let claims = TestClaims {
            sub: "test_user".to_string(),
            exp: 0, // No expiration
        };
        
        let token = encode(
            &Header::new(ALGORITHM),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap();
        
        let mut validation = Validation::new(ALGORITHM);
        validation.validate_exp = false; // Disable expiration validation
        
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
        let decoded_claims = decoded.unwrap().claims;
        assert_eq!(decoded_claims.sub, "test_user");
        assert_eq!(decoded_claims.exp, 0);
    }

    #[test]
    fn test_token_with_future_expiration() {
        let far_future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + (365 * 24 * 3600); // 1 year in future
        
        let claims = TestClaims {
            sub: "test_user".to_string(),
            exp: far_future as usize,
        };
        
        let token = encode(
            &Header::new(ALGORITHM),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap();
        
        let validation = Validation::new(ALGORITHM);
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        );
        
        assert!(decoded.is_ok());
    }
}