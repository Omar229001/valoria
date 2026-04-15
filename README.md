# 🚗 Valoria - Plateforme de Cotation de Véhicules d'Occasion

Une plateforme moderne de cotation et d'évaluation de véhicules d'occasion utilisant **web scraping**, **machine learning** et une **architecture microservices**.

- 🏗️ **Architecture**: API Gateway + 4 services backend + Frontend React
- 🦀 **Backend**: Rust (performance) + Python (scraping)
- 💾 **Data**: PostgreSQL + Redis + RabbitMQ
- 📦 **Deploy**: Docker Compose + Terraform (DigitalOcean)

---

## 📋 Table des matières

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [APIs](#apis)
- [Configuration](#configuration)
- [Développement](#développement)
- [Déploiement](#déploiement)
- [Contributing](#contributing)

---

## 🚀 Quick Start

### Prérequis
- Docker & Docker Compose
- Rust 1.77+ (si développement local)
- Python 3.11+ (si développement local)
- Node.js 18+ (si développement frontend)

### Démarrer tout en local

```bash
# 1. Cloner le repo
git clone https://github.com/valoria/valoria.git
cd valoria

# 2. Copier la configuration d'exemple
cp .env.example .env

# 3. Lancer tous les services
make dev

# 4. Vérifier que tout fonctionne
make ps
make logs

# Frontend: http://localhost:4000
# API Gateway: http://localhost:8080
# API Docs: http://localhost:8080/docs (coming soon)
```

### Arrêter tout

```bash
make down        # Arrête les conteneurs
make clean       # Arrête + supprime les volumes
```

### Logs et débogage

```bash
make logs                 # Tous les logs en temps réel
make logs-gateway         # Logs du Gateway seulement
make logs-user            # Logs du User Service seulement
make logs-scraper         # Logs du Scraper Service seulement
make logs-cotation        # Logs du Cotation Service seulement
make logs-pricing         # Logs du Pricing Service seulement
```

---

## 🏛️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React 18)                       │
│                    http://localhost:4000                         │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│            API Gateway (Rust + Axum)                            │
│  - JWT validation & routing                                     │
│  - Reverse proxy                                                │
│  - CORS handling                                                │
│              http://localhost:8080                              │
└──┬──────────────────┬──────────────────┬───────────────────────┘
   │                  │                  │
   ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌────────────────┐
│ User Service │  │Cotation Svc  │  │Scraper Service │
│  (Rust)      │  │  (Rust)      │  │  (Python)      │
│ Auth & User  │  │  Quotation   │  │ Web Scraping   │
│  :8004       │  │   :8003      │  │    :8002       │
└──────────────┘  └──────────────┘  └────────────────┘
   │ (DB)             │ (DB)             │ (DB)
   └─────────┬────────┴─────────┬────────┘
             ▼                  ▼
        ┌─────────────────────────────┐
        │      PostgreSQL DB           │
        │    (port 5434)               │
        └─────────────────────────────┘

Async Jobs: RabbitMQ (port 5672)
Cache: Redis (port 6379)
```

### Services

| Service | Port | Rôle | Stack |
|---------|------|------|-------|
| **API Gateway** | 8080 | Proxy & auth validation | Rust + Axum |
| **User Service** | 8004 | Authentification, gestion utilisateurs | Rust + SQLx |
| **Cotation Service** | 8003 | Évaluation des véhicules, cotation | Rust |
| **Scraper Service** | 8002 | Web scraping des annonces | Python + FastAPI + Playwright |
| **Pricing Service** | 8001 | Calcul des prix, analyse | Rust |
| **Frontend** | 4000 | Interface utilisateur | React 18 + TypeScript + Vite |

---

## 📡 APIs

### Endpoints disponibles

#### 🔐 Authentification (publique)
- `POST /api/auth/register` - Créer un compte
- `POST /api/auth/login` - Se connecter
- `GET /api/auth/me` - Infos utilisateur actuel (JWT requis)

#### 💰 Cotation (JWT requis)
- `POST /api/cotation` - Créer une nouvelle cotation
- `GET /api/cotation/history` - Historique des cotations

#### 📊 Listings (JWT requis)
- `GET /api/listings?brand=Renault&model=Clio&year_min=2020&year_max=2023` - Récupérer les annonces

#### 🕷️ Scraping (JWT requis)
- `POST /api/scrape/sync` - Lancer le scraping synchrone
- `GET /api/listings` - Récupérer les résultats scrapés

### Documentation API interactive

Les APIs REST sont documentées avec OpenAPI/Swagger :

```bash
# Lancer les services
make dev

# Accéder à Swagger UI
open http://localhost:8080/docs

# JSON OpenAPI
http://localhost:8080/openapi.json
```

### Exemples de requêtes

#### 1. S'inscrire et se connecter

```bash
# S'inscrire
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice Dupont",
    "email": "alice@example.com",
    "password": "secure_password_123"
  }'

# Réponse:
# {
#   "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
#   "user": {
#     "id": 1,
#     "name": "Alice Dupont",
#     "email": "alice@example.com",
#     "role": "user"
#   }
# }

# Se connecter
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "secure_password_123"
  }'
```

#### 2. Récupérer les infos utilisateur

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $TOKEN"
```

#### 3. Créer une cotation

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X POST http://localhost:8080/api/cotation \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Renault",
    "model": "Clio",
    "year": 2021,
    "mileage": 45000,
    "condition": "good"
  }'
```

#### 4. Lancer le scraping

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X POST http://localhost:8080/api/scrape/sync \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Renault",
    "model": "Clio",
    "year_min": 2020,
    "year_max": 2023
  }'
```

#### 5. Récupérer les annonces

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X GET "http://localhost:8080/api/listings?brand=Renault&model=Clio&year_min=2020&year_max=2023" \
  -H "Authorization: Bearer $TOKEN"
```

---

## ⚙️ Configuration

### Variables d'environnement

Créer un fichier `.env` à la racine du projet:

```bash
# API Gateway
JWT_SECRET=your_super_secret_key_here_change_in_prod
USER_SERVICE_URL=http://user-service:8000
COTATION_SERVICE_URL=http://cotation-service:8000
SCRAPER_SERVICE_URL=http://scraper-service:8000
PORT=8080

# User Service
DATABASE_URL=postgresql://valoria:valoria_dev@postgres:5432/valoria
JWT_SECRET=your_super_secret_key_here_change_in_prod

# Scraper Service
DATABASE_URL=postgresql://valoria:valoria_dev@postgres:5432/valoria
RABBITMQ_URL=amqp://valoria:valoria_dev@rabbitmq:5672

# Cotation Service
DATABASE_URL=postgresql://valoria:valoria_dev@postgres:5432/valoria
PRICING_SERVICE_URL=http://pricing-service:8000
SCRAPER_SERVICE_URL=http://scraper-service:8000
RABBITMQ_URL=amqp://valoria:valoria_dev@rabbitmq:5672

# PostgreSQL
POSTGRES_USER=valoria
POSTGRES_PASSWORD=valoria_dev
POSTGRES_DB=valoria
```

Voir `.env.example` pour l'exemple complet.

### Connexion à la base de données

```bash
# Depuis le host (si services Docker tournent)
psql -h localhost -p 5434 -U valoria -d valoria

# Password: valoria_dev
```

---

## 🛠️ Développement

### Structure du projet

```
valoria/
├── services/
│   ├── api-gateway/           # Reverse proxy + JWT validation
│   │   ├── src/main.rs
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   ├── user-service/          # Authentification & utilisateurs
│   │   ├── src/main.rs
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   ├── cotation-service/      # Cotation des véhicules
│   │   ├── src/main.rs
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   ├── scraper-service/       # Web scraping
│   │   ├── src/
│   │   │   ├── main.py
│   │   │   ├── database.py
│   │   │   ├── schemas.py
│   │   │   └── scrapers/
│   │   ├── requirements.txt
│   │   └── Dockerfile
│   └── pricing-service/       # Calcul des prix
│       ├── src/main.rs
│       ├── Cargo.toml
│       └── Dockerfile
├── frontend/                   # Interface React
│   ├── src/
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   └── ...
│   ├── package.json
│   └── Dockerfile
├── infra/                      # Infrastructure as Code
│   └── terraform/
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
└── docker-compose.yml         # Orchestration locale
```

### Développement local sans Docker

#### Rust Services

```bash
cd services/user-service
cargo build
cargo run

# Tests
cargo test
```

#### Python Services

```bash
cd services/scraper-service
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
uvicorn src.main:app --reload

# Tests
pytest
```

#### Frontend

```bash
cd frontend
npm install
npm run start     # Mode développement
npm run build     # Production build
npm run preview   # Prévisualiser le build
```

### Tests

```bash
# Tests de tous les services (à implémenter)
make test

# Tests Rust spécifiques
cd services/user-service && cargo test
cd services/api-gateway && cargo test

# Tests Python spécifiques
cd services/scraper-service && pytest
```

### Linting

```bash
# Rust
cargo clippy

# Python
flake8 services/scraper-service/
black services/scraper-service/

# Frontend
cd frontend && npm run lint
```

---

## 🚀 Déploiement

### Sur DigitalOcean (avec Terraform)

```bash
cd infra/terraform

# Configurer vos credentials
export DIGITALOCEAN_TOKEN="your_token"

# Planifier le déploiement
terraform plan

# Appliquer la configuration
terraform apply

# Détruire les ressources (attention!)
terraform destroy
```

### Variables Terraform importants

```hcl
# infra/terraform/terraform.tfvars
region              = "nyc3"
environment         = "production"
app_name            = "valoria"
replicas            = 3
database_size       = "db-s-2vcpu-4gb"
```

### Déploiement manual sur un serveur

```bash
# 1. SSH sur le serveur
ssh root@your_server

# 2. Cloner le repo
git clone https://github.com/valoria/valoria.git
cd valoria

# 3. Copier les variables de prod
scp .env.production root@your_server:/app/.env

# 4. Lancer avec docker compose
make dev

# 5. Setup DNS, SSL avec Let's Encrypt
# (À configurer via Nginx reverse proxy)
```

---

## 📚 Ressources & Documentation

- [Axum (Rust web framework)](https://github.com/tokio-rs/axum)
- [FastAPI (Python web framework)](https://fastapi.tiangolo.com/)
- [SQLx (Async SQL toolkit)](https://github.com/launchbadge/sqlx)
- [React Documentation](https://react.dev)
- [Docker Documentation](https://docs.docker.com)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)

---

## 🤝 Contributing

Les contributions sont les bienvenues! 

1. Fork le repo
2. Crée une branche (`git checkout -b feature/amazing-feature`)
3. Commit tes changements (`git commit -m 'Add amazing feature'`)
4. Push vers la branche (`git push origin feature/amazing-feature`)
5. Ouvre une Pull Request

### Checklist avant de soumettre

- [ ] Code testé localement
- [ ] Tests ajoutés/passants
- [ ] Linting passant (`cargo clippy`, `black`, `eslint`)
- [ ] Documentation mise à jour

---

## 📄 License

Ce projet est sous license MIT. Voir `LICENSE` pour plus de détails.

---

## 💬 Support

- 📧 Email: support@valoria.dev
- 💬 Discord: [Rejoindre le serveur](https://discord.gg/valoria)
- 📖 Docs: https://docs.valoria.dev

---

**Made with ❤️ by the Valoria team**