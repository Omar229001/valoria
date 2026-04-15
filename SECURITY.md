# 🔐 Security Guide

Sécurité et gestion des secrets pour Valoria.

---

## 📋 Table des matières

1. [Environment Variables](#environment-variables)
2. [Secrets Management](#secrets-management)
3. [Security Checklist](#security-checklist)
4. [JWT Security](#jwt-security)
5. [Database Security](#database-security)
6. [API Security](#api-security)

---

## ⚙️ Environment Variables

### Required Variables

**TOUS les services doivent avoir:**

```bash
# MANDATORY - Must be set in production
ENVIRONMENT=production              # development|staging|production
JWT_SECRET=<long-random-string>     # Generated secret, NEVER use default
DATABASE_URL=postgresql://...       # Connection string

# Optional but important
PORT=8000                           # Service port
LOG_LEVEL=info                      # Logging level
```

### Generation Secrets

```bash
# Generate a secure JWT secret (32 bytes = 256 bits)
openssl rand -base64 32

# Generate a random password
openssl rand -base64 24

# On Windows
$bytes = New-Object byte[] 32
[Security.Cryptography.RNGCryptoServiceProvider]::new().GetBytes($bytes)
[Convert]::ToBase64String($bytes)
```

### Local Development

**`.env` (LOCAL ONLY, never commit):**

```bash
ENVIRONMENT=development
JWT_SECRET=dev_secret_only_for_local_testing_change_in_prod
DATABASE_URL=postgresql://valoria:valoria_dev@localhost:5434/valoria
PORT=8000
```

**`.env.production` (EXAMPLE, replace with actual values):**

```bash
ENVIRONMENT=production
JWT_SECRET=<generated-via-openssl>
DATABASE_URL=postgresql://prod_user:prod_password@prod-db.example.com:5432/valoria_prod
PORT=8080
```

### Environment-Specific Files

Three config files:

| File | Purpose | Location | Committed |
|------|---------|----------|-----------|
| `.env.example` | Template | Git ✓ | YES - document all vars |
| `.env` | Local dev | Git ✗ | NO - local only |
| `.env.production` | Prod config | Git ✗ | NO - secrets inside |

---

## 🔑 Secrets Management

### What NOT to do

```rust
// ❌ WRONG: Hardcoded secret
let jwt_secret = "dev_secret_change_in_prod";

// ❌ WRONG: Comment with password
// username: admin, password: password123

// ❌ WRONG: Committed .env file
// DATABASE_URL=postgresql://user:p@ssw0rd@...
```

### What TO do

```rust
// ✅ CORRECT: Load from environment
let jwt_secret = env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set in environment");

// ✅ CORRECT: Validate it's not default
if jwt_secret == "dev_secret_change_in_prod" {
    panic!("Using default JWT secret - SECURITY RISK!");
}

// ✅ CORRECT: Use config module
use valoria::config::Config;
let config = Config::from_env();  // Validates and panics on error
```

### Secrets in Production

**Never commit secrets to git:**

```bash
# Check what you're about to commit
git diff --cached

# Verify no secrets in the diff
grep -E "password|secret|token|key" <diff>

# If you accidentally committed:
git rm --cached .env.production
git commit --amend
```

### Store Secrets in Production

Options (pick one):

1. **Docker Secrets** (Swarm/Kubernetes)
```bash
# In docker-compose.yml
services:
  api-gateway:
    environment:
      JWT_SECRET_FILE: /run/secrets/jwt_secret
    secrets:
      - jwt_secret

secrets:
  jwt_secret:
    external: true
```

2. **Vault** (HashiCorp)
```rust
// Fetch secret at runtime from Vault
let secret = vault_client.read("secret/data/jwt_secret")?;
```

3. **AWS Secrets Manager**
```rust
let secret = aws_secretsmanager::get_secret("jwt_secret").await?;
```

4. **Environment variables** (via CI/CD)
```yaml
# In GitHub Actions
- name: Deploy
  env:
    JWT_SECRET: ${{ secrets.JWT_SECRET }}
    DATABASE_URL: ${{ secrets.DATABASE_URL }}
  run: docker compose up
```

---

## ✅ Security Checklist

### Before Each Deployment

- [ ] No `.env` file committed
- [ ] All secrets in environment variables
- [ ] JWT_SECRET is NOT "dev_secret_change_in_prod"
- [ ] Database URL uses strong password
- [ ] CORS properly configured (not permissive in prod)
- [ ] HTTPS enabled (TLS certificates)
- [ ] Rate limiting enabled
- [ ] Input validation on all endpoints
- [ ] SQL injection protections (parameterized queries)
- [ ] XSS protections (CSP headers)
- [ ] Secrets rotated regularly

### Code Review

- [ ] No passwords in comments
- [ ] No API keys in code
- [ ] No default secrets
- [ ] ENV variables properly validated
- [ ] Error messages don't leak secrets
- [ ] Logs don't contain sensitive data

---

## 🔐 JWT Security

### Token Generation

```rust
// In user-service
let now = Utc::now();
let claims = Claims {
    sub: user.id,                              // User ID
    email: user.email.clone(),                 // Email (ok to expose)
    name: user.name.clone(),                   // Name (ok to expose)
    role: user.role.clone(),                   // Role
    exp: (now + Duration::hours(24)).timestamp(),  // Expiration
    iat: now.timestamp(),                      // Issued at
};

let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;
```

### Token Validation

```rust
// In api-gateway
fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;  // ✅ Verify expiration
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?.claims
}
```

### Token Storage (Client)

**❌ WRONG:**
```javascript
// Never in localStorage - XSS vuln!
localStorage.setItem('token', jwt_token);
```

**✅ CORRECT:**
```javascript
// HTTP-only cookie (set by server)
// Browser auto-sends with requests
// Protected against XSS
```

---

## 🗄️ Database Security

### Connection String

```bash
# ✅ CORRECT: Strong password, TLS
postgresql://user:strong_p@ssw0rd@db.prod.com:5432/valoria?sslmode=require

# ❌ WRONG: Weak password
postgresql://user:12345@db.prod.com:5432/valoria

# ❌ WRONG: No SSL
postgresql://user:pass@db.prod.com:5432/valoria?sslmode=disable
```

### SQL Injection Protection

```rust
// ✅ CORRECT: Parameterized query (SQLx)
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE email = $1"  // $1 is placeholder
)
.bind(&email)  // Bind parameter safely
.fetch_one(&pool)
.await?;

// ❌ WRONG: String concatenation
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
// Vulnerable if email = "' OR '1'='1"
```

### Credentials

```bash
# Development credentials
POSTGRES_USER=valoria
POSTGRES_PASSWORD=valoria_dev  # Only for development

# Production credentials (NEVER commit)
POSTGRES_USER=prod_user_randomized
POSTGRES_PASSWORD=$(openssl rand -base64 32)  # Very strong
```

---

## 🔒 API Security

### CORS Configuration

```rust
// ❌ WRONG: Permissive CORS in production
let cors = CorsLayer::permissive();  // Allows any origin!

// ✅ CORRECT: Restricted CORS
let cors = CorsLayer::very_restrictive()
    .allow_origin("https://example.com".parse()?)
    .allow_methods([Method::GET, Method::POST]);
```

### Input Validation

```rust
// ❌ WRONG: No validation
async fn register(Json(req): Json<RegisterRequest>) -> Response {
    // req.email could be anything!
}

// ✅ CORRECT: Validate with Pydantic/Validator
#[derive(Deserialize, Validator)]
struct RegisterRequest {
    #[validate(length(min = 2, max = 100))]
    name: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}
```

### Error Responses

```rust
// ❌ WRONG: Leaks implementation details
(500, "Database error: connection refused at 10.0.1.5:5432")

// ✅ CORRECT: Generic error message
(500, "Internal server error")

// ✅ Log the actual error (not in response)
eprintln!("[ERROR] DB connection failed: {}", error);
```

### Rate Limiting

```rust
// ✅ CORRECT: Implement rate limiting
middleware::RateLimitingLayer::new(
    RateLimit {
        requests_per_minute: 100,
        burst_size: 10,
    }
)
```

---

## 🚨 Incident Response

### If a Secret is Leaked

1. **IMMEDIATELY:** Rotate the secret
2. **Generate new:** `openssl rand -base64 32`
3. **Update in:**
   - Production environment
   - All dependent services
   - Docker secrets/Vault
4. **Invalidate old:** Delete old token/key
5. **Monitor:** Watch for unauthorized access
6. **Notify:** Alert your team/users if needed

### Command Sequence

```bash
# 1. Generate new secret
NEW_SECRET=$(openssl rand -base64 32)
echo "New secret: $NEW_SECRET"

# 2. Update production (example with AWS)
aws secretsmanager update-secret \
  --secret-id jwt_secret \
  --secret-string "$NEW_SECRET"

# 3. Restart services with new secret
# (Kubernetes, Docker, etc.)

# 4. Monitor logs for errors
docker logs valoria-api-gateway | grep "error"
```

---

## 📚 Security Best Practices

### 1. Principle of Least Privilege
- Database user has minimal permissions
- Services only access what they need
- SSH keys for specific users only

### 2. Defense in Depth
- Multiple layers of security
- Encryption in transit (TLS)
- Encryption at rest (if applicable)
- Input validation
- Output encoding

### 3. Regular Audits
- Review who has access
- Rotate secrets regularly (every 90 days)
- Check for leaked secrets with `gitleaks`
- Scan dependencies for vulnerabilities

### 4. Monitoring & Logging
- Log authentication attempts
- Alert on failed logins (>5 in 5 min)
- Monitor for unusual API usage
- Never log passwords/tokens

---

## 🔧 Tools

### Local Development

```bash
# Generate secure secrets
openssl rand -base64 32

# Check for secrets in code
gitleaks detect --source . -v

# Check for hardcoded passwords
grep -r "password\|secret\|token\|key" \
  --include="*.rs" --include="*.py" \
  services/ | grep -v test | grep -v doc
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for secrets
if git diff --cached | grep -E "dev_secret|valoria_dev|password"; then
    echo "❌ ERROR: Found hardcoded secrets in commit"
    exit 1
fi

echo "✅ No secrets detected"
```

---

## 📖 Further Reading

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [RFC 7519 - JWT](https://tools.ietf.org/html/rfc7519)
- [12 Factor App](https://12factor.net/) - especially Config
- [HashiCorp Vault](https://www.vaultproject.io/)
- [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/)

---

## ✅ Quick Checklist for New Features

Before committing:

- [ ] No hardcoded passwords/keys
- [ ] Environment variables for all secrets
- [ ] Input validation implemented
- [ ] Error messages don't leak info
- [ ] .env files in .gitignore
- [ ] Logs don't contain sensitive data
- [ ] Code reviewed by at least one person
