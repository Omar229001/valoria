# 📡 Valoria API Documentation

Documentation complète de tous les endpoints REST de la plateforme Valoria.

**Base URL:** `http://localhost:8080/api` (développement)

**Authentication:** Token JWT en header `Authorization: Bearer <token>`

---

## 📋 Table des matières

1. [Authentification](#authentification)
2. [Cotation](#cotation)
3. [Listings](#listings)
4. [Scraping](#scraping)
5. [Erreurs](#erreurs)
6. [Rate Limiting](#rate-limiting)

---

## 🔐 Authentification

### POST /auth/register

Créer un nouveau compte utilisateur.

**Requête:**

```bash
POST /api/auth/register
Content-Type: application/json

{
  "name": "Alice Dupont",
  "email": "alice@example.com",
  "password": "SecurePassword123!"
}
```

**Paramètres:**

| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `name` | string | ✅ | Nom complet (2-100 caractères) |
| `email` | string | ✅ | Email unique (format valide) |
| `password` | string | ✅ | Mot de passe (min 8 caractères) |

**Réponse (201 Created):**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": 1,
    "name": "Alice Dupont",
    "email": "alice@example.com",
    "role": "user",
    "created_at": "2024-04-15T09:47:00Z"
  }
}
```

**Erreurs:**

| Code | Message | Cause |
|------|---------|-------|
| 409 | Email déjà utilisé | L'email est déjà enregistré |
| 400 | Validation failed | Email invalide ou mot de passe trop court |
| 500 | Database error | Erreur serveur |

**Exemple avec curl:**

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice Dupont",
    "email": "alice@example.com",
    "password": "SecurePassword123!"
  }'
```

---

### POST /auth/login

Se connecter et obtenir un token JWT.

**Requête:**

```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "SecurePassword123!"
}
```

**Paramètres:**

| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `email` | string | ✅ | Email de l'utilisateur |
| `password` | string | ✅ | Mot de passe |

**Réponse (200 OK):**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": 1,
    "name": "Alice Dupont",
    "email": "alice@example.com",
    "role": "user"
  }
}
```

**Token JWT structure:**

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": 1,
    "email": "alice@example.com",
    "name": "Alice Dupont",
    "role": "user",
    "exp": 1713091020,
    "iat": 1713004620
  }
}
```

Le token expire après **24 heures**. Utilisez-le dans l'en-tête `Authorization: Bearer <token>` pour les requêtes authentifiées.

**Erreurs:**

| Code | Erreur | Cause |
|------|--------|-------|
| 401 | Email ou mot de passe incorrect | Email inexistant ou mot de passe faux |
| 500 | Database error | Erreur serveur |

**Exemple avec curl:**

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "SecurePassword123!"
  }' | jq -r '.token'
```

---

### GET /auth/me

Récupérer les infos de l'utilisateur actuellement connecté.

**Requête:**

```bash
GET /api/auth/me
Authorization: Bearer <your_jwt_token>
```

**Réponse (200 OK):**

```json
{
  "id": 1,
  "name": "Alice Dupont",
  "email": "alice@example.com",
  "role": "user",
  "created_at": "2024-04-15T09:47:00Z"
}
```

**Erreurs:**

| Code | Erreur | Cause |
|------|--------|-------|
| 401 | Token manquant | Pas d'en-tête `Authorization` |
| 401 | Token invalide ou expiré | Token malformé ou expiré |
| 404 | Utilisateur introuvable | L'utilisateur n'existe plus |

**Exemple avec curl:**

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $TOKEN"
```

---

## 💰 Cotation

### POST /cotation

Créer une nouvelle cotation de véhicule.

**Requête:**

```bash
POST /api/cotation
Authorization: Bearer <your_jwt_token>
Content-Type: application/json

{
  "brand": "Renault",
  "model": "Clio",
  "year": 2021,
  "mileage": 45000,
  "fuel": "essence",
  "transmission": "manuelle",
  "condition": "bon",
  "comments": "Très bon état, un propriétaire"
}
```

**Paramètres:**

| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `brand` | string | ✅ | Marque du véhicule (ex: Renault, Peugeot) |
| `model` | string | ✅ | Modèle (ex: Clio, 308) |
| `year` | integer | ✅ | Année (1980-2024) |
| `mileage` | integer | ✅ | Kilométrage (0+) |
| `fuel` | string | ✅ | Carburant (essence, diesel, électrique, hybride) |
| `transmission` | string | ✅ | Transmission (manuelle, automatique) |
| `condition` | string | ✅ | État général (excellent, bon, moyen, mauvais) |
| `comments` | string | ❌ | Notes supplémentaires |

**Réponse (201 Created):**

```json
{
  "cotation_id": 42,
  "brand": "Renault",
  "model": "Clio",
  "year": 2021,
  "mileage": 45000,
  "condition": "bon",
  "ml_estimated_price": 12500,
  "estimated_price": 12800,
  "min_price": 11500,
  "max_price": 14200,
  "confidence": 0.85,
  "method": "market_analysis + ml_model",
  "market": {
    "listings_count": 234,
    "median_price": 12650,
    "min_price": 9800,
    "max_price": 18900
  },
  "created_at": "2024-04-15T09:47:00Z"
}
```

**Erreurs:**

| Code | Erreur | Cause |
|------|--------|-------|
| 400 | Validation failed | Paramètres invalides |
| 401 | Token invalide | Token expiré ou malformé |
| 500 | Pricing service unavailable | Service de pricing indisponible |

**Exemple avec curl:**

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
    "fuel": "essence",
    "transmission": "manuelle",
    "condition": "bon"
  }' | jq
```

---

### GET /cotation/history

Récupérer l'historique des cotations de l'utilisateur.

**Requête:**

```bash
GET /api/cotation/history?limit=10&offset=0
Authorization: Bearer <your_jwt_token>
```

**Paramètres de query:**

| Paramètre | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 10 | Nombre de résultats (max 100) |
| `offset` | integer | 0 | Pagination offset |
| `brand` | string | - | Filtrer par marque |
| `model` | string | - | Filtrer par modèle |
| `year_from` | integer | - | Année minimum |
| `year_to` | integer | - | Année maximum |

**Réponse (200 OK):**

```json
{
  "total": 42,
  "limit": 10,
  "offset": 0,
  "data": [
    {
      "cotation_id": 42,
      "brand": "Renault",
      "model": "Clio",
      "year": 2021,
      "estimated_price": 12800,
      "created_at": "2024-04-15T09:47:00Z"
    },
    {
      "cotation_id": 41,
      "brand": "Peugeot",
      "model": "308",
      "year": 2020,
      "estimated_price": 15200,
      "created_at": "2024-04-14T14:32:00Z"
    }
  ]
}
```

**Exemple avec curl:**

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
curl -X GET "http://localhost:8080/api/cotation/history?limit=20" \
  -H "Authorization: Bearer $TOKEN" | jq
```

---

## 📊 Listings

### GET /listings

Récupérer les annonces (résultats du scraping).

**Requête:**

```bash
GET /api/listings?brand=Renault&model=Clio&year_min=2020&year_max=2023&limit=50
Authorization: Bearer <your_jwt_token>
```

**Paramètres de query:**

| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `brand` | string | ✅ | Marque |
| `model` | string | ✅ | Modèle |
| `year_min` | integer | ✅ | Année minimum |
| `year_max` | integer | ✅ | Année maximum |
| `limit` | integer | ❌ | Limite (default 50, max 1000) |
| `offset` | integer | ❌ | Pagination (default 0) |
| `sort` | string | ❌ | Tri (price_asc, price_desc, mileage_asc, date_desc) |

**Réponse (200 OK):**

```json
{
  "total": 234,
  "limit": 50,
  "offset": 0,
  "data": [
    {
      "id": 1001,
      "brand": "Renault",
      "model": "Clio",
      "year": 2021,
      "mileage": 32500,
      "price": 12500,
      "fuel": "essence",
      "transmission": "manuelle",
      "city": "Paris",
      "source": "lacentrale",
      "url": "https://www.lacentrale.fr/...",
      "scraped_at": "2024-04-15T08:00:00Z"
    },
    {
      "id": 1002,
      "brand": "Renault",
      "model": "Clio",
      "year": 2022,
      "mileage": 18900,
      "price": 14200,
      "fuel": "essence",
      "transmission": "automatique",
      "city": "Lyon",
      "source": "leboncoin",
      "url": "https://www.leboncoin.fr/...",
      "scraped_at": "2024-04-15T08:30:00Z"
    }
  ]
}
```

**Exemple avec curl:**

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
curl -X GET "http://localhost:8080/api/listings?brand=Renault&model=Clio&year_min=2020&year_max=2023&limit=20" \
  -H "Authorization: Bearer $TOKEN" | jq
```

---

## 🕷️ Scraping

### POST /scrape/sync

Lancer le web scraping synchrone sur tous les sites.

**Requête:**

```bash
POST /api/scrape/sync
Authorization: Bearer <your_jwt_token>
Content-Type: application/json

{
  "brand": "Renault",
  "model": "Clio",
  "year_min": 2020,
  "year_max": 2023,
  "sources": "all"
}
```

**Paramètres:**

| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `brand` | string | ✅ | Marque |
| `model` | string | ✅ | Modèle |
| `year_min` | integer | ✅ | Année minimum |
| `year_max` | integer | ✅ | Année maximum |
| `sources` | string | ❌ | Sites à scraper (all, lacentrale, leboncoin, autoscout24) |

**Réponse (200 OK):**

```json
[
  {
    "source": "lacentrale",
    "listings_found": 127,
    "listings_saved": 125,
    "status": "success"
  },
  {
    "source": "leboncoin",
    "listings_found": 89,
    "listings_saved": 87,
    "status": "success"
  },
  {
    "source": "autoscout24",
    "listings_found": 45,
    "listings_saved": 44,
    "status": "success"
  }
]
```

**⚠️ Temps d'exécution:** 30-60 secondes selon le nombre de résultats.

**Exemple avec curl:**

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
curl -X POST http://localhost:8080/api/scrape/sync \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Renault",
    "model": "Clio",
    "year_min": 2020,
    "year_max": 2023,
    "sources": "all"
  }' | jq
```

---

## ⚠️ Erreurs

Tous les endpoints retournent des erreurs au format JSON :

```json
{
  "error": "Description de l'erreur",
  "status": 400,
  "path": "/api/auth/login",
  "timestamp": "2024-04-15T09:47:00Z"
}
```

### Codes d'erreur courants

| Code | Nom | Cause |
|------|------|-------|
| 400 | Bad Request | Paramètres invalides ou malformés |
| 401 | Unauthorized | Token manquant, invalide ou expiré |
| 403 | Forbidden | Permissions insuffisantes |
| 404 | Not Found | Resource inexistante |
| 409 | Conflict | Resource en doublon (ex: email déjà utilisé) |
| 429 | Too Many Requests | Rate limit dépassée |
| 500 | Internal Server Error | Erreur serveur |
| 503 | Service Unavailable | Service temporairement indisponible |

---

## 🚦 Rate Limiting

**Limites actuelles (à implémenter):**

- Authentification: 5 requêtes par minute par IP
- Scraping: 1 requête par minute par utilisateur
- Autres endpoints: 100 requêtes par minute par utilisateur

**Headers de réponse:**

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1713091020
```

Quand le rate limit est atteint:

```json
HTTP 429 Too Many Requests

{
  "error": "Rate limit exceeded. Try again in 60 seconds",
  "retry_after": 60
}
```

---

## 📝 Notes

- Tous les timestamps sont en UTC (ISO 8601)
- Les montants sont en EUR
- Les kilométrages sont en km
- Les années sont au format YYYY (ex: 2024)

---

## 🔗 Ressources

- [OpenAPI Specification](https://swagger.io/specification/)
- [JWT Documentation](https://jwt.io/introduction)
- [HTTP Status Codes](https://httpwg.org/specs/rfc9110.html#status.codes)
