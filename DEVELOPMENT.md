# 🛠️ Development Guide

Guide complet pour développer sur Valoria localement.

---

## 📋 Table des matières

1. [Quick Start](#quick-start)
2. [Services Setup](#services-setup)
3. [Database](#database)
4. [Debugging](#debugging)
5. [Common Tasks](#common-tasks)
6. [Troubleshooting](#troubleshooting)

---

## 🚀 Quick Start

### Option 1: Avec Docker (recommandé)

```bash
# Clone le repo
git clone https://github.com/valoria/valoria.git
cd valoria

# Copie config
cp .env.example .env

# Lance tout
make dev

# Vérifier
make ps

# Logs
make logs
```

Ensuite:
- Frontend: http://localhost:4000
- API: http://localhost:8080
- DB: localhost:5434 (psql)

### Option 2: Local (sans Docker)

Tu dois avoir:
- Rust 1.77+
- Python 3.11+
- Node.js 18+
- PostgreSQL running localement
- Redis running localement
- RabbitMQ running localement

```bash
# Terminal 1: PostgreSQL
brew services start postgresql

# Terminal 2: Redis
redis-server

# Terminal 3: RabbitMQ
rabbitmq-server

# Terminal 4: User Service
cd services/user-service
cargo run

# Terminal 5: Cotation Service
cd services/cotation-service
cargo run

# Terminal 6: Scraper Service
cd services/scraper-service
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
uvicorn src.main:app --reload

# Terminal 7: Pricing Service
cd services/pricing-service
cargo run

# Terminal 8: API Gateway
cd services/api-gateway
cargo run

# Terminal 9: Frontend
cd frontend
npm install
npm run start
```

---

## 📦 Services Setup

### User Service

```bash
cd services/user-service

# Build
cargo build

# Run
cargo run

# Run in release mode (optimized)
cargo run --release

# Tests
cargo test

# Watch for changes and re-run tests
cargo test -- --nocapture --test-threads=1 (watch)

# Check code without building
cargo check

# Format code
cargo fmt

# Linting
cargo clippy
cargo clippy --fix
```

**Endpoints:**
- `GET /health`
- `POST /auth/register`
- `POST /auth/login`
- `GET /auth/me`

**Vérifier que ça fonctionne:**

```bash
# Health check
curl http://localhost:8000/health

# Register
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test User",
    "email": "test@example.com",
    "password": "password123"
  }'
```

---

### Scraper Service

```bash
cd services/scraper-service

# Setup
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# Install deps
pip install -r requirements.txt

# Run
uvicorn src.main:app --reload
# API docs: http://localhost:8000/docs

# Run specific endpoint
uvicorn src.main:app --reload --port 8001

# Tests
pytest -v

# Coverage
pytest --cov=src

# Lint
black src/
flake8 src/

# Type check
mypy src/
```

**Endpoints:**
- `GET /health`
- `POST /scrape` - Async scraping
- `POST /scrape/sync` - Sync scraping
- `GET /listings` - Get results
- `GET /debug/autoscout24` - Debug endpoint

**Vérifier:**

```bash
curl http://localhost:8000/health

# Test scraping
curl -X POST http://localhost:8000/scrape/sync \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Renault",
    "model": "Clio",
    "year_min": 2020,
    "year_max": 2023
  }'
```

---

### API Gateway

```bash
cd services/api-gateway

# Build
cargo build

# Run
cargo run

# Tests
cargo test

# Lint
cargo clippy
```

**Vérifier:**

```bash
curl http://localhost:8080/health
```

---

### Frontend

```bash
cd frontend

# Install deps
npm install

# Dev server (hot reload)
npm run start

# Build for production
npm run build

# Preview production build
npm run preview

# Linting
npm run lint

# Tests (if configured)
npm test
```

Accéder à: http://localhost:4000

---

## 🗄️ Database

### Connexion

```bash
# Via Docker container
docker exec -it valoria-postgres psql -U valoria -d valoria

# Via host (si port mappé 5434)
psql -h localhost -p 5434 -U valoria -d valoria
# Password: valoria_dev
```

### Voir les tables

```sql
\dt  -- List tables

-- Users
SELECT * FROM users;

-- Car listings
SELECT * FROM car_listings LIMIT 5;

-- Cotations
SELECT * FROM cotations LIMIT 5;
```

### Reset database

```bash
# Depuis Docker
docker exec valoria-postgres psql -U valoria -d valoria -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

# Ou via make
make clean  # Arrête et supprime les volumes
make dev    # Redémarre (BD vierge)
```

### Backups

```bash
# Backup
docker exec valoria-postgres pg_dump -U valoria -d valoria > backup.sql

# Restore
docker exec -i valoria-postgres psql -U valoria -d valoria < backup.sql
```

---

## 🐛 Debugging

### Logs

```bash
# Tous les services
make logs

# Service spécifique
make logs-gateway
make logs-user
make logs-scraper
make logs-cotation
make logs-pricing

# Avec tail
docker compose logs -f api-gateway

# X dernières lignes
docker compose logs --tail=50 api-gateway
```

### Rust Debugging

```bash
# Avec RUST_BACKTRACE
RUST_BACKTRACE=1 cargo run

# Full backtrace
RUST_BACKTRACE=full cargo run

# Debug symbols
cargo build
lldb target/debug/user-service
```

### Python Debugging

```bash
# Avec pdb (breakpoint)
# Dans le code:
import pdb; pdb.set_trace()

# Puis run:
python -m pdb src/main.py

# Ou avec pytest
pytest --pdb src/tests/test_scraper.py
```

### Network Debugging

```bash
# Voir les requêtes entre services
docker compose logs -f | grep "Gateway\|Service"

# Test connectivity
docker exec valoria-api-gateway curl http://user-service:8000/health

# Check port availability
netstat -an | grep 8080
```

### VS Code Debugging

**Pour Rust:**

Fichier `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Launch User Service",
      "cargo": {
        "args": [
          "build",
          "--bin=user_service",
          "--package=user_service"
        ]
      },
      "args": [],
      "cwd": "${workspaceFolder}/services/user-service"
    }
  ]
}
```

---

## 📝 Common Tasks

### Ajouter une dépendance

#### Rust

```bash
cd services/user-service

# Add dependency
cargo add tokio --features full

# Check Cargo.toml was updated
git diff Cargo.toml

# Build to verify
cargo build
```

#### Python

```bash
cd services/scraper-service

# Activate venv
source venv/bin/activate

# Install package
pip install requests

# Update requirements
pip freeze > requirements.txt

# Verify
cat requirements.txt | grep requests
```

#### JavaScript/Node

```bash
cd frontend

# Install package
npm install axios

# Update package.json
cat package.json | grep axios

# Verify it works
npm test
```

### Ajouter une nouvelle route

#### Rust + Axum

```rust
// services/user-service/src/main.rs

// 1. Créer le handler
async fn get_profile(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    // implementation
    Json(json!({"profile": "data"})).into_response()
}

// 2. Ajouter la route
let app = Router::new()
    .route("/health", get(health))
    .route("/auth/register", post(register))
    .route("/auth/login", post(login))
    .route("/auth/me", get(me))
    .route("/profile", get(get_profile))  // NEW
    .layer(Extension(state))
    .layer(CorsLayer::permissive());
```

#### Python + FastAPI

```python
# services/scraper-service/src/main.py

@app.get("/listings-by-price")
def get_listings_by_price(min_price: int, max_price: int):
    """Get listings filtered by price range."""
    return get_listings_by_price_range(min_price, max_price)

# Access: GET /listings-by-price?min_price=10000&max_price=20000
```

### Ajouter une migration DB

**Pour PostgreSQL (SQLx):**

```bash
# SQLx compile-time verification
cd services/user-service

# Ajouter la migration
cargo sqlx migrate add -r create_user_table

# Edit the migration file
vim migrations/20240415_create_user_table.sql

# Run migrations
cargo sqlx migrate run

# Revert (for testing)
cargo sqlx migrate revert
```

**Exemple migration:**

```sql
-- migrations/20240415_create_user_table.sql

-- Up
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role VARCHAR(20) DEFAULT 'user',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Down
DROP TABLE users;
```

---

## ❌ Troubleshooting

### "Connection refused" on port 8080

```bash
# Vérifier que le gateway est bien lancé
docker compose ps

# Ou relancer
make restart
```

### "Database connection failed"

```bash
# Vérifier que PostgreSQL est up
docker exec valoria-postgres pg_isready -U valoria -d valoria

# Vérifier les credentials en .env
cat .env | grep DATABASE_URL

# Check logs
docker compose logs postgres
```

### "JWT token invalid"

```bash
# Vérifier que JWT_SECRET est le même partout
grep JWT_SECRET .env

# Tester le token
TOKEN="your_token_here"
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $TOKEN" \
  -v
```

### Ports already in use

```bash
# Trouver quel process utilise le port
lsof -i :8080

# Ou tuer tous les containers
docker compose down
make clean
```

### "ModuleNotFoundError" in Python

```bash
# Vérifier le venv est activé
which python  # Doit montrer le chemin venv/bin/python

# Réinstall deps
pip install -r requirements.txt

# Vérifier PYTHONPATH
export PYTHONPATH="${PYTHONPATH}:/path/to/valoria/services/scraper-service"
```

### Build issues

```bash
# Nettoyer tout
cargo clean  # Rust
rm -rf node_modules && npm install  # Node

# Rebuild
cargo build
npm run build

# Force rebuild
cargo build --release
```

---

## 🎯 Workflow recommandé

1. **Fork le repo**
2. **Clone ton fork**
3. **Create une branche**: `git checkout -b feature/my-feature`
4. **Setup local**:
   ```bash
   cp .env.example .env
   make dev
   ```
5. **Code & test**
6. **Commit** avec messages clairs
7. **Push** vers ton fork
8. **Open PR**

---

## 📚 Ressources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [FastAPI Tutorial](https://fastapi.tiangolo.com/tutorial/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)

**Besoin d'aide?**
- Ouvre une Issue
- Demande dans les Discussions
- Check le CONTRIBUTING.md
