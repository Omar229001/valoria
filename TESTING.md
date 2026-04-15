# 🧪 Testing Guide

Guide complet des tests pour Valoria.

---

## 📋 Table des matières

1. [Testing Strategy](#testing-strategy)
2. [Rust Tests](#rust-tests)
3. [Python Tests](#python-tests)
4. [Frontend Tests](#frontend-tests)
5. [Integration Tests](#integration-tests)
6. [Running Tests](#running-tests)
7. [Coverage](#coverage)

---

## 🎯 Testing Strategy

**Pyramide de tests:**

```
        /\         E2E Tests (1-5%)
       /  \        Slow, high-level
      /────\
     /      \
    /────────\     Integration Tests (15-30%)
   /          \    Medium speed
  /────────────\
 /              \
/────────────────\ Unit Tests (70-80%)
Fast, isolated    Highly focused
```

**Targets:**
- ✅ Unit tests: 80%+
- ✅ Integration tests: 15%+
- ✅ E2E tests: 5%+
- ✅ **Overall coverage: 70%+**

---

## 🦀 Rust Tests

### Unit Tests

**Format:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid_email"));
    }

    #[test]
    fn test_hash_password() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_panic_on_invalid_jwt() {
        validate_jwt("invalid_token", "secret").unwrap();
    }
}
```

**Où:**
- `services/user-service/src/main.rs` - Tests d'auth
- `services/api-gateway/src/main.rs` - Tests du gateway
- `services/cotation-service/src/main.rs` - Tests cotation

### Async Tests

```rust
#[tokio::test]
async fn test_register_user() {
    let pool = setup_test_db().await;
    
    let req = RegisterRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    
    let result = register_user(&pool, req).await;
    
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

### Running Rust Tests

```bash
# Tests du service
cd services/user-service

# Tous les tests
cargo test

# Test spécifique
cargo test test_validate_email

# Avec output
cargo test -- --nocapture

# Single-threaded (pour les tests qui modifient l'état global)
cargo test -- --test-threads=1

# Tests en release mode (plus rapide)
cargo test --release

# Voir les tests sans les run
cargo test -- --list
```

---

## 🐍 Python Tests

### Unit Tests avec pytest

**Format:**

```python
# services/scraper-service/src/tests/test_scraper.py

import pytest
from src.scrapers.lacentrale import LaCentraleScraper
from src.database import save_listings

def test_lacentrale_scraper_init():
    scraper = LaCentraleScraper()
    assert scraper is not None
    assert scraper.base_url == "https://www.lacentrale.fr"

@pytest.mark.asyncio
async def test_scrape_listings():
    scraper = LaCentraleScraper()
    listings = await scraper.search("Renault", "Clio", 2020, 2023)
    
    assert isinstance(listings, list)
    if listings:
        assert listings[0].brand == "Renault"
        assert listings[0].year >= 2020

def test_database_save():
    listings = [
        {"brand": "Renault", "model": "Clio", "year": 2021, "price": 12000},
    ]
    count = save_listings(listings)
    assert count == 1

@pytest.fixture
def test_db():
    """Setup test database"""
    db = init_test_db()
    yield db
    cleanup_test_db()

def test_with_fixture(test_db):
    """Test using fixture"""
    assert test_db is not None
```

### Running Python Tests

```bash
cd services/scraper-service

# Tous les tests
pytest

# Verbose output
pytest -v

# Test spécifique
pytest tests/test_scraper.py::test_lacentrale_scraper_init

# Avec print output
pytest -s

# Stop on first failure
pytest -x

# Tests matching a pattern
pytest -k "test_scrape"

# Run with timeout
pytest --timeout=300

# Show slowest tests
pytest --durations=10

# Coverage report
pytest --cov=src --cov-report=html
```

### Fixtures pytest

```python
import pytest
from unittest.mock import Mock, patch

@pytest.fixture
def mock_browser():
    """Mock Playwright browser"""
    mock = Mock()
    return mock

@pytest.fixture
def test_scraper(mock_browser):
    """Create scraper with mocked browser"""
    scraper = LaCentraleScraper()
    scraper.browser = mock_browser
    return scraper

def test_with_mock(test_scraper):
    # Use mocked scraper
    test_scraper.browser.goto.assert_called()
```

---

## ⚛️ Frontend Tests

### Unit Tests avec Vitest

**Format:**

```typescript
// frontend/src/__tests__/utils.test.ts

import { describe, it, expect } from 'vitest';
import { isValidEmail, formatPrice } from '../utils';

describe('Email validation', () => {
  it('validates correct emails', () => {
    expect(isValidEmail('test@example.com')).toBe(true);
  });

  it('rejects invalid emails', () => {
    expect(isValidEmail('not_an_email')).toBe(false);
  });
});

describe('Price formatting', () => {
  it('formats prices correctly', () => {
    expect(formatPrice(12500)).toBe('12 500,00 €');
  });
});
```

### Component Tests

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LoginForm } from '../LoginForm';

describe('LoginForm', () => {
  it('renders input fields', () => {
    render(<LoginForm onLogin={() => {}} />);
    
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
  });

  it('handles form submission', async () => {
    const handleLogin = vi.fn();
    render(<LoginForm onLogin={handleLogin} />);
    
    const user = userEvent.setup();
    await user.type(screen.getByLabelText(/email/i), 'test@example.com');
    await user.type(screen.getByLabelText(/password/i), 'password123');
    await user.click(screen.getByRole('button', { name: /login/i }));
    
    expect(handleLogin).toHaveBeenCalled();
  });
});
```

### Running Frontend Tests

```bash
cd frontend

# Tous les tests
npm test

# Watch mode
npm test -- --watch

# Coverage
npm test -- --coverage

# Single test file
npm test -- LoginForm

# Debug
npm test -- --inspect-brk
```

---

## 🔗 Integration Tests

### API Integration Tests

**Format (Rust):**

```rust
#[tokio::test]
async fn test_full_auth_flow() {
    let client = setup_test_client().await;
    
    // Register
    let register_resp = client
        .post("/api/auth/register")
        .json(&json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(register_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = register_resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    
    // Login
    let login_resp = client
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(login_resp.status(), StatusCode::OK);
    
    // Get current user
    let me_resp = client
        .get("/api/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(me_resp.status(), StatusCode::OK);
}
```

### API Testing avec curl

```bash
# Register
TOKEN=$(curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","email":"test@example.com","password":"pass123"}' \
  | jq -r '.token')

# Use token
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $TOKEN" | jq

# Create cotation
curl -X POST http://localhost:8080/api/cotation \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Renault",
    "model": "Clio",
    "year": 2021,
    "mileage": 45000,
    "fuel": "essence",
    "transmission": "manuelle",
    "condition": "bon"
  }' | jq
```

---

## 🏃 Running Tests

### Run All Tests

```bash
# Rust
cd services/user-service && cargo test
cd services/api-gateway && cargo test
cd services/cotation-service && cargo test

# Python
cd services/scraper-service && pytest

# Frontend
cd frontend && npm test
```

### Make Commands (à ajouter)

```bash
# Dans Makefile:
.PHONY: test test-rust test-python test-frontend

test:
	cargo test -C services/user-service
	cargo test -C services/api-gateway
	pytest services/scraper-service/
	npm test -C frontend

test-rust:
	cargo test -C services/user-service
	cargo test -C services/api-gateway
	cargo test -C services/cotation-service

test-python:
	pytest services/scraper-service/

test-frontend:
	npm test -C frontend
```

---

## 📊 Coverage

### Rust Coverage

```bash
# Avec tarpaulin
cargo tarpaulin --out Html

# Avec llvm-cov
cargo llvm-cov --html
```

### Python Coverage

```bash
cd services/scraper-service

# Generate coverage report
pytest --cov=src --cov-report=html

# View report
open htmlcov/index.html
```

### Frontend Coverage

```bash
cd frontend

# Generate coverage
npm test -- --coverage

# View report
open coverage/index.html
```

---

## ✅ Pre-commit Testing

**Setup git hook:**

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running tests..."

# Rust tests
cargo test -C services/user-service
cargo test -C services/api-gateway

# Python tests
pytest services/scraper-service/ -q

# Frontend tests
npm test -C frontend -- --run

echo "All tests passed! ✓"
```

---

## 🐛 Debugging Tests

### Rust Debugging

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Run single test
cargo test test_name -- --exact

# List all tests
cargo test -- --list
```

### Python Debugging

```bash
# Use pdb
pytest -s tests/test_file.py -k test_name

# Print debug info
pytest -s --tb=short

# Stop on first failure
pytest -x
```

### Frontend Debugging

```bash
# Run in debug mode
npm test -- --debug

# Watch mode
npm test -- --watch

# Open in browser
npm test -- --reporter=verbose
```

---

## 📚 Testing Best Practices

### ✅ Do

- Write tests close to code they test
- Use descriptive test names
- Test one thing per test
- Use fixtures for setup/teardown
- Keep tests fast
- Test edge cases

### ❌ Don't

- Test implementation details
- Make tests depend on each other
- Use actual external services (mock them)
- Sleep in tests
- Ignore test failures

---

## 🚀 Next Steps

1. **Add tests to CI/CD pipeline**
2. **Setup code coverage badges**
3. **Add mutation testing**
4. **Setup performance benchmarks**
5. **Add load testing**
