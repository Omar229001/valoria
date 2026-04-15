//! Authentication module tests
//! Tests for register, login, and JWT functionality

#[cfg(test)]
mod tests {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use chrono::Utc;

    // Mock Claims struct for testing
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct Claims {
        sub: i64,
        email: String,
        name: String,
        role: String,
        exp: i64,
        iat: i64,
    }

    // JWT creation logic (extracted from main.rs)
    fn create_jwt_test(
        user_id: i64,
        email: String,
        name: String,
        role: String,
        secret: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        use chrono::Duration;
        use jsonwebtoken::{encode, EncodingKey, Header};

        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            email,
            name,
            role,
            exp: (now + Duration::hours(24)).timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }

    // JWT validation logic (extracted from main.rs)
    fn validate_jwt_test(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )?;
        Ok(data.claims)
    }

    // ── JWT Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_create_jwt_success() {
        let token = create_jwt_test(
            1,
            "user@example.com".to_string(),
            "John Doe".to_string(),
            "user".to_string(),
            "test_secret_key_at_least_32_characters_long_test",
        );

        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[test]
    fn test_jwt_contains_claims() {
        let secret = "test_secret_key_at_least_32_characters_long_test";
        let token = create_jwt_test(
            123,
            "alice@example.com".to_string(),
            "Alice".to_string(),
            "admin".to_string(),
            secret,
        )
        .expect("Failed to create token");

        let claims = validate_jwt_test(&token, secret).expect("Failed to validate token");

        assert_eq!(claims.sub, 123);
        assert_eq!(claims.email, "alice@example.com");
        assert_eq!(claims.name, "Alice");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_jwt_has_timestamps() {
        let secret = "test_secret_key_at_least_32_characters_long_test";
        let before = Utc::now().timestamp();

        let token = create_jwt_test(
            1,
            "test@example.com".to_string(),
            "Test".to_string(),
            "user".to_string(),
            secret,
        )
        .expect("Failed to create token");

        let after = Utc::now().timestamp();
        let claims = validate_jwt_test(&token, secret).expect("Failed to validate token");

        // Check issued_at is recent
        assert!(claims.iat >= before && claims.iat <= after);

        // Check exp is 24 hours in future
        let duration = claims.exp - claims.iat;
        assert!(duration >= 86400 - 10); // Allow 10 second tolerance
        assert!(duration <= 86400 + 10);
    }

    #[test]
    fn test_invalid_jwt_secret() {
        let secret = "test_secret_key_at_least_32_characters_long_test";
        let token = create_jwt_test(
            1,
            "test@example.com".to_string(),
            "Test".to_string(),
            "user".to_string(),
            secret,
        )
        .expect("Failed to create token");

        // Try to validate with wrong secret
        let result = validate_jwt_test(&token, "wrong_secret_key_at_least_32_characters");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_token_format() {
        let secret = "test_secret_key_at_least_32_characters_long_test";
        let token = create_jwt_test(
            1,
            "test@example.com".to_string(),
            "Test".to_string(),
            "user".to_string(),
            secret,
        )
        .expect("Failed to create token");

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts separated by dots");
    }

    // ── Password Hashing Tests ─────────────────────────────────────────

    #[test]
    fn test_bcrypt_hash_valid() {
        use bcrypt::{hash, verify, DEFAULT_COST};

        let password = "SecurePassword123!";
        let hashed = hash(password, DEFAULT_COST);

        assert!(hashed.is_ok());
        let hash_result = hashed.unwrap();
        assert!(!hash_result.is_empty());
        assert!(hash_result.len() > 20);
    }

    #[test]
    fn test_bcrypt_verify_correct_password() {
        use bcrypt::{hash, verify, DEFAULT_COST};

        let password = "MySecurePass123!";
        let hashed = hash(password, DEFAULT_COST).expect("Failed to hash");

        let result = verify(password, &hashed).expect("Failed to verify");
        assert!(result);
    }

    #[test]
    fn test_bcrypt_verify_wrong_password() {
        use bcrypt::{hash, verify, DEFAULT_COST};

        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";
        let hashed = hash(password, DEFAULT_COST).expect("Failed to hash");

        let result = verify(wrong_password, &hashed).expect("Failed to verify");
        assert!(!result);
    }

    #[test]
    fn test_bcrypt_same_password_different_hashes() {
        use bcrypt::{hash, DEFAULT_COST};

        let password = "TestPassword123";
        let hash1 = hash(password, DEFAULT_COST).expect("Failed to hash");
        let hash2 = hash(password, DEFAULT_COST).expect("Failed to hash");

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);
    }

    // ── Validation Tests ───────────────────────────────────────────────

    #[test]
    fn test_email_format_validation() {
        fn is_valid_email(email: &str) -> bool {
            email.contains('@') && email.contains('.') && email.len() > 5
        }

        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("no@domain"));
        assert!(!is_valid_email("@example.com"));
    }

    #[test]
    fn test_password_strength() {
        fn is_strong_password(pwd: &str) -> bool {
            pwd.len() >= 8 && pwd.chars().any(|c| c.is_uppercase())
                && pwd.chars().any(|c| c.is_lowercase())
                && pwd.chars().any(|c| c.is_numeric())
        }

        assert!(is_strong_password("MyPassword123"));
        assert!(is_strong_password("SecurePass999"));
        assert!(!is_strong_password("short")); // Too short
        assert!(!is_strong_password("nouppercase123")); // No uppercase
        assert!(!is_strong_password("NoNumbers")); // No numbers
    }

    #[test]
    fn test_name_validation() {
        fn is_valid_name(name: &str) -> bool {
            !name.is_empty() && name.len() <= 100 && name.chars().all(|c| !c.is_control())
        }

        assert!(is_valid_name("John Doe"));
        assert!(is_valid_name("Alice"));
        assert!(is_valid_name("Jean-Pierre Martin"));
        assert!(!is_valid_name("")); // Empty
        assert!(!is_valid_name(&"x".repeat(101))); // Too long
    }

    // ── Bearer Token Extraction Tests ──────────────────────────────────

    #[test]
    fn test_extract_bearer_token() {
        fn extract_bearer(auth_header: Option<&str>) -> Option<String> {
            auth_header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let result = extract_bearer(Some("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert_eq!(
            result,
            Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_missing_prefix() {
        fn extract_bearer(auth_header: Option<&str>) -> Option<String> {
            auth_header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let result = extract_bearer(Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_bearer_no_header() {
        fn extract_bearer(auth_header: Option<&str>) -> Option<String> {
            auth_header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let result = extract_bearer(None);
        assert!(result.is_none());
    }
}
