//! API Gateway JWT validation tests
//! Tests for JWT token validation and claims extraction

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    struct Claims {
        sub: i64,
        email: String,
        name: String,
        role: String,
        exp: i64,
        iat: i64,
    }

    fn create_jwt_for_testing(
        user_id: i64,
        email: &str,
        name: &str,
        role: &str,
        secret: &str,
    ) -> Result<String, String> {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            exp: (now + Duration::hours(24)).timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| e.to_string())
    }

    fn validate_jwt_for_testing(token: &str, secret: &str) -> Result<Claims, String> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
    }

    // ── JWT Validation Tests ───────────────────────────────────────────

    #[test]
    fn test_valid_jwt_passes_validation() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "user@test.com", "Test User", "user", secret)
            .expect("Failed to create token");

        let result = validate_jwt_for_testing(&token, secret);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_signature_fails_validation() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let wrong_secret = "wrong_secret_key_at_least_32_characters";

        let token = create_jwt_for_testing(1, "user@test.com", "Test User", "user", secret)
            .expect("Failed to create token");

        let result = validate_jwt_for_testing(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_token_fails_validation() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "user@test.com", "Test User", "user", secret)
            .expect("Failed to create token");

        // Tamper with the payload
        let parts: Vec<&str> = token.split('.').collect();
        let mut tampered = parts[0].to_string();
        tampered.push_str(".tampered");
        tampered.push('.');
        tampered.push_str(parts[2]);

        let result = validate_jwt_for_testing(&tampered, secret);
        assert!(result.is_err());
    }

    // ── Claims Extraction Tests ────────────────────────────────────────

    #[test]
    fn test_extract_user_id_from_claims() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(42, "user@test.com", "Test User", "user", secret)
            .expect("Failed to create token");

        let claims = validate_jwt_for_testing(&token, secret).expect("Failed to validate");
        assert_eq!(claims.sub, 42);
    }

    #[test]
    fn test_extract_email_from_claims() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "alice@example.com", "Alice", "admin", secret)
            .expect("Failed to create token");

        let claims = validate_jwt_for_testing(&token, secret).expect("Failed to validate");
        assert_eq!(claims.email, "alice@example.com");
    }

    #[test]
    fn test_extract_role_from_claims() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "admin@test.com", "Admin User", "admin", secret)
            .expect("Failed to create token");

        let claims = validate_jwt_for_testing(&token, secret).expect("Failed to validate");
        assert_eq!(claims.role, "admin");
    }

    // ── Expiration Tests ───────────────────────────────────────────────

    #[test]
    fn test_jwt_expiration_timestamp() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "user@test.com", "User", "user", secret)
            .expect("Failed to create token");

        let claims = validate_jwt_for_testing(&token, secret).expect("Failed to validate");
        let now = Utc::now().timestamp();

        // Should not be expired (exp should be 24 hours in future)
        assert!(claims.exp > now);
        assert!(claims.exp > now + 86300); // ~24 hours - 100 seconds
    }

    #[test]
    fn test_jwt_issued_at_timestamp() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let before = Utc::now().timestamp();

        let token = create_jwt_for_testing(1, "user@test.com", "User", "user", secret)
            .expect("Failed to create token");

        let after = Utc::now().timestamp();
        let claims = validate_jwt_for_testing(&token, secret).expect("Failed to validate");

        // iat should be very recent
        assert!(claims.iat >= before);
        assert!(claims.iat <= after);
    }

    // ── Header Injection Tests ─────────────────────────────────────────

    #[test]
    fn test_inject_user_headers_from_claims() {
        fn create_user_headers(
            user_id: i64,
            email: &str,
            name: &str,
            role: &str,
        ) -> Vec<(String, String)> {
            vec![
                ("X-User-Id".to_string(), user_id.to_string()),
                ("X-User-Email".to_string(), email.to_string()),
                ("X-User-Name".to_string(), name.to_string()),
                ("X-User-Role".to_string(), role.to_string()),
            ]
        }

        let headers = create_user_headers(1, "user@test.com", "Test User", "admin");

        assert_eq!(headers[0].0, "X-User-Id");
        assert_eq!(headers[0].1, "1");
        assert_eq!(headers[1].0, "X-User-Email");
        assert_eq!(headers[1].1, "user@test.com");
    }

    // ── Token Format Tests ─────────────────────────────────────────────

    #[test]
    fn test_jwt_has_three_parts() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "user@test.com", "User", "user", secret)
            .expect("Failed to create token");

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_jwt_parts_not_empty() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let token = create_jwt_for_testing(1, "user@test.com", "User", "user", secret)
            .expect("Failed to create token");

        let parts: Vec<&str> = token.split('.').collect();
        assert!(!parts[0].is_empty(), "Header part is empty");
        assert!(!parts[1].is_empty(), "Payload part is empty");
        assert!(!parts[2].is_empty(), "Signature part is empty");
    }

    // ── Bearer Token Extraction from Headers ───────────────────────────

    #[test]
    fn test_extract_bearer_token_from_authorization_header() {
        fn extract_jwt_from_header(header: Option<&str>) -> Option<String> {
            header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let header = Some("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature");
        let token = extract_jwt_from_header(header);

        assert_eq!(
            token,
            Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature".to_string())
        );
    }

    #[test]
    fn test_missing_bearer_prefix_returns_none() {
        fn extract_jwt_from_header(header: Option<&str>) -> Option<String> {
            header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let header = Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature");
        let token = extract_jwt_from_header(header);

        assert!(token.is_none());
    }

    #[test]
    fn test_missing_authorization_header_returns_none() {
        fn extract_jwt_from_header(header: Option<&str>) -> Option<String> {
            header
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }

        let header: Option<&str> = None;
        let token = extract_jwt_from_header(header);

        assert!(token.is_none());
    }
}
