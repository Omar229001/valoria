# 🏛️ Valoria Architecture

Guide détaillé de l'architecture microservices de Valoria.

---

## 📊 Diagramme global

```
┌─────────────────────────────────────────────────────────┐
│                   CLIENTS                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Web Browser  │  │ Mobile App   │  │ API Client   │  │
│  │ (React 18)   │  │ (iOS/Android)│  │ (External)   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
        ┌────────────────────▼────────────────────┐
        │  API Gateway (Rust + Axum)             │
        │  Port: 8080                            │
        │  ✓ JWT validation                      │
        │  ✓ Request routing                     │
        │  ✓ CORS handling                       │
        │  ✓ Rate limiting                       │
        └───┬────────────────┬────────────────────┘
            │                │
        ┌───▼────────┐   ┌───▼────────────────────┐
        │ /api/auth  │   │ /api/{other}           │
        └───┬────────┘   └───────┬────────────────┘
            │                    │
    ┌───────▼──────────┐    ┌────┴──────┬─────────────┐
    │ User Service     │    │            │             │
    │ Port: 8004       │    │            │             │
    │ (Rust + Axum)    │    │            │             │
    └──────┬───────────┘    │            │             │
           │                │            │             │
      ┌────▼────────────────▼──────┐ ┌──▼──────────┐ ┌─▼──────────────┐
      │ Cotation Service           │ │Scraper Svc  │ │Pricing Service │
      │ Port: 8003                 │ │Port: 8002   │ │Port: 8001      │
      │ (Rust + Axum)              │ │(Python +    │ │(Rust + Axum)   │
      │                            │ │FastAPI)     │ │                │
      └────┬────────────────────────┘ └──┬──────────┘ └─┬──────────────┘
           │                             │             │
           └─────────┬───────────────────┴─────────────┘
                     │
          ┌──────────▼─────────────────────────────┐
          │      PostgreSQL (DB)                   │
          │      Port: 5434                        │
          │      ✓ Users                           │
          │      ✓ Cotations                       │
          │      ✓ Listings                        │
          │      ✓ Pricing data                    │
          └────────────────────────────────────────┘

          ┌──────────────────────┐
          │ Redis (Cache)        │
          │ Port: 6379           │
          │ ✓ Session tokens     │
          │ ✓ Rate limits        │
          └──────────────────────┘

          ┌──────────────────────┐
          │ RabbitMQ (Queue)     │
          │ Port: 5672           │
          │ ✓ Async scraping     │
          │ ✓ Pricing jobs       │
          └──────────────────────┘
```

---

## 🔄 Communication entre services

### Gateway → Services

L'API Gateway agit comme **reverse proxy** :

1. **Requête reçue** → `/api/cotation/history`
2. **JWT valide** ✓ (sinon: 401 Unauthorized)
3. **Route déterminée** → Service de cotation
4. **Headers ajoutés** → `X-User-Id`, `X-User-Email` (du JWT)
5. **Requête forwardée** → `http://cotation-service:8000/cotation/history`
6. **Réponse retournée** → Client

```rust
// Exemple: Gateway routes les endpoints
if path.starts_with("/api/auth") {
    target = user_service_url;
    requires_auth = false;
} else if path.starts_with("/api/cotation") {
    target = cotation_service_url;
    requires_auth = true;
}
```

### Inter-service communication

Services s'appellent directement par HTTP (même réseau Docker):

```
Cotation Service
    ↓ (appel HTTP)
Pricing Service
    ↓ (requête: GET /pricing/estimate)
Réponse JSON
    ↓
Cotation Service ✓
```

Exemple:

```rust
// Dans Cotation Service
let pricing_resp = reqwest::get(
    &format!("{}/pricing/estimate?brand=Renault&model=Clio", 
        pricing_service_url)
).await?;
```

### Async Jobs

RabbitMQ gère les tâches longues (scraping, recalcul de prix):

```
POST /api/scrape/sync
    ↓
Scraper Service enqueue dans RabbitMQ
    ↓
Background worker process RabbitMQ messages
    ↓
Résultats sauvegardés en DB
    ↓
Client poll GET /listings pour résultats
```

---

## 📦 Services détaillés

### 1️⃣ API Gateway (Rust + Axum)

**Responsabilités:**
- Authentification JWT
- Routage des requêtes
- CORS
- Rate limiting
- Logging

**Architecture:**

```rust
Router::new()
  .route("/health", get(health))
  .route("/api/auth/*", proxy)        // → User Service
  .route("/api/cotation/*", proxy)    // → Cotation Service
  .route("/api/listings/*", proxy)    // → Scraper Service
  .layer(CorsLayer::permissive())
```

**Dépendances clés:**
- `axum` - Framework web
- `jsonwebtoken` - JWT validation
- `reqwest` - HTTP client pour proxy
- `tower-http` - CORS, compression

---

### 2️⃣ User Service (Rust + SQLx)

**Responsabilités:**
- Authentification (register/login)
- Gestion des utilisateurs
- Génération JWT

**Endpoints:**
- `POST /auth/register`
- `POST /auth/login`
- `GET /auth/me`

**DB Schema:**

```sql
CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  role VARCHAR(20) DEFAULT 'user',
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Dépendances clés:**
- `axum` - Framework web
- `sqlx` - Database access
- `bcrypt` - Password hashing
- `jsonwebtoken` - JWT generation

---

### 3️⃣ Scraper Service (Python + FastAPI)

**Responsabilités:**
- Web scraping de sites d'occasion
- Parsing des annonces
- Sauvegarde en DB

**Sources:**
- LasCentrale.fr
- LeBonCoin.fr
- AutoScout24.fr

**Endpoints:**
- `POST /scrape` - Async (background task)
- `POST /scrape/sync` - Sync (attend résultats)
- `GET /listings` - Récupère les annonces
- `GET /debug/autoscout24` - Debug endpoint

**DB Schema:**

```sql
CREATE TABLE car_listings (
  id SERIAL PRIMARY KEY,
  brand VARCHAR(100),
  model VARCHAR(100),
  year INTEGER,
  mileage INTEGER,
  fuel VARCHAR(50),
  transmission VARCHAR(50),
  price FLOAT,
  city VARCHAR(100),
  source VARCHAR(100),
  url TEXT UNIQUE,
  scraped_at TIMESTAMP DEFAULT NOW()
);
```

**Dépendances clés:**
- `fastapi` - Framework web
- `playwright` - Browser automation
- `psycopg2` - PostgreSQL client
- `pydantic` - Data validation

---

### 4️⃣ Cotation Service (Rust + Axum)

**Responsabilités:**
- Évaluation des véhicules
- Calcul de cotations
- Appel au Pricing Service

**Endpoints:**
- `POST /cotation` - Créer cotation
- `GET /cotation/history` - Historique utilisateur

**Flux:**

```
Requête POST /cotation
  ↓
Valider les paramètres
  ↓
Appeler Pricing Service pour estimation ML
  ↓
Requêter Scraper Service pour market data
  ↓
Calculer price range (min/max/médian)
  ↓
Combiner résultats
  ↓
Sauvegarder cotation en DB
  ↓
Retourner résultats
```

**DB Schema:**

```sql
CREATE TABLE cotations (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT REFERENCES users(id),
  brand VARCHAR(100),
  model VARCHAR(100),
  year INTEGER,
  mileage INTEGER,
  estimated_price FLOAT,
  confidence FLOAT,
  method VARCHAR(100),
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

### 5️⃣ Pricing Service (Rust + Axum)

**Responsabilités:**
- Calcul du prix selon caractéristiques
- ML model pour estimation
- Analyse de marché

**Endpoints:**
- `POST /pricing/estimate`
- `GET /pricing/stats`

**Modèle:**

```
Inputs: brand, model, year, mileage, fuel, condition
  ↓
ML Model (regression)
  ↓
Output: estimated_price + confidence
```

---

## 🗄️ Data Flow

### Authentification

```
1. User POST /api/auth/login
           ↓
2. Gateway → User Service
           ↓
3. User Service: verify password + generate JWT
           ↓
4. Return token to client
           ↓
5. Client stores token (localStorage)
           ↓
6. Subsequent requests: Authorization: Bearer <token>
```

### Cotation Creation

```
1. User POST /api/cotation + JWT
           ↓
2. Gateway validates JWT
           ↓
3. Gateway forwards to Cotation Service
           ↓
4. Cotation Service gets X-User-Id from headers
           ↓
5. Cotation Service calls:
   - Pricing Service: /pricing/estimate
   - Scraper Service: /listings?brand=X&model=Y
           ↓
6. Combine results + calculate confidence
           ↓
7. Save to DB (cotations table)
           ↓
8. Return response to client
```

### Scraping Job

```
1. User POST /api/scrape/sync
           ↓
2. Gateway → Scraper Service
           ↓
3. Scraper launches Playwright for each source
           ↓
4. Parse HTML, extract listings
           ↓
5. Deduplicate (by URL unique index)
           ↓
6. Save to car_listings table
           ↓
7. Return count of new listings
```

---

## 🔐 Sécurité

### JWT Token Structure

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": 1,              // User ID
    "email": "alice@example.com",
    "name": "Alice Dupont",
    "role": "user",
    "exp": 1713091020,     // Expiration
    "iat": 1713004620      // Issued at
  },
  "signature": "..."
}
```

**Valisation:**
- Vérifier signature avec `JWT_SECRET`
- Vérifier expiration
- Si valide: extraire user info → headers

### Authentification Flow

```
GET /api/cotation/history + Bearer token
           ↓
Gateway: extract token from Authorization header
           ↓
Verify signature: decode(token, JWT_SECRET)
           ↓
Check expiration: now < exp?
           ↓
✓ Valid: add X-User-Id header
✗ Invalid: return 401 Unauthorized
           ↓
Forward to service with X-User-Id
```

---

## 🚀 Déploiement

### Local (Docker Compose)

```bash
make dev  # Démarre tous les conteneurs
```

Services:
- API Gateway: `http://localhost:8080`
- User Service: `http://localhost:8004`
- Cotation Service: `http://localhost:8003`
- Scraper Service: `http://localhost:8002`
- Pricing Service: `http://localhost:8001`
- Frontend: `http://localhost:4000`

### Production (Terraform + DigitalOcean)

```
DigitalOcean App Platform
├── Frontend (Static)
├── API Gateway (Container)
├── User Service (Container)
├── Cotation Service (Container)
├── Scraper Service (Container)
├── Pricing Service (Container)
├── PostgreSQL (Managed)
├── Redis (Managed)
└── RabbitMQ (Container)
```

---

## 📈 Scaling

### Horizontal Scaling

Réplicas multiples derrière un load balancer:

```
Load Balancer
├── User Service (replica 1)
├── User Service (replica 2)
├── User Service (replica 3)
├── Cotation Service (replica 1)
├── Cotation Service (replica 2)
└── ...
```

Partagent la même BD + Redis.

### Vertical Scaling

Augmenter CPU/RAM des services individuels.

---

## 🔍 Monitoring & Observability

**À implémenter:**

- Prometheus metrics
- Structured logging (JSON)
- Distributed tracing (Jaeger)
- Health checks (`GET /health`)
- Readiness probes (DB connectivity)

---

## 📚 Tech Stack Justification

| Composant | Choix | Justification |
|-----------|-------|---------------|
| Gateway | Rust + Axum | Perf, type-safe, low latency |
| Auth/API | Rust + SQLx | Type-safe, zero-cost abstractions |
| Scraper | Python + FastAPI | Rapide à dev, Playwright mature |
| Cache | Redis | Fast KV store, sessions, rate limits |
| Queue | RabbitMQ | Reliable, AMQP protocol |
| DB | PostgreSQL | ACID, JSON support, mature |
| Frontend | React + Vite | Modern, fast build, ecosystem |

---

## 🔗 Ressources

- [Axum documentation](https://docs.rs/axum/)
- [SQLx documentation](https://github.com/launchbadge/sqlx)
- [FastAPI documentation](https://fastapi.tiangolo.com/)
- [PostgreSQL documentation](https://www.postgresql.org/docs/)
- [JWT.io](https://jwt.io/)
