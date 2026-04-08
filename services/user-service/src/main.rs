use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;

// ── Modèles ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password_hash: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    email: String,
    name: String,
    role: String,
    exp: i64,
    iat: i64,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    jwt_secret: String,
}

// ── Helpers JWT ───────────────────────────────────────────────────────────────

fn create_jwt(user: &User, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        exp: (now + Duration::hours(24)).timestamp(),
        iat: now.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn validate_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(data.claims)
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok", "service": "user-service"}))
}

async fn register(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    // Vérifier si l'email est déjà pris
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(&req.email)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(false);

    if exists {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Email déjà utilisé"})),
        )
            .into_response();
    }

    // Hasher le mot de passe
    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Hash error: {}", e)})),
            )
                .into_response();
        }
    };

    // Insérer l'utilisateur
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, password_hash)
         VALUES ($1, $2, $3)
         RETURNING id, name, email, password_hash, role",
    )
    .bind(&req.name)
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await;

    match user {
        Ok(u) => match create_jwt(&u, &state.jwt_secret) {
            Ok(token) => (
                StatusCode::CREATED,
                Json(json!({"token": token, "user": u})),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("JWT error: {}", e)})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("DB error: {}", e)})),
        )
            .into_response(),
    }
}

async fn login(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role FROM users WHERE email = $1",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Email ou mot de passe incorrect"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            )
                .into_response();
        }
    };

    match verify(&req.password, &user.password_hash) {
        Ok(true) => {}
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Email ou mot de passe incorrect"})),
            )
                .into_response();
        }
    }

    match create_jwt(&user, &state.jwt_secret) {
        Ok(token) => Json(json!({"token": token, "user": user})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("JWT error: {}", e)})),
        )
            .into_response(),
    }
}

async fn me(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Token manquant. Header: Authorization: Bearer <token>"})),
            )
                .into_response();
        }
    };

    let claims = match validate_jwt(&token, &state.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Token invalide ou expiré"})),
            )
                .into_response();
        }
    };

    match sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(u)) => Json(u).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Utilisateur introuvable"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_in_prod".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    // Connexion DB avec retry
    let pool = loop {
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                println!("[user-service] DB connectée ✓");
                break p;
            }
            Err(e) => {
                eprintln!("[user-service] DB non disponible: {}. Retry dans 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    };

    // Créer la table users si elle n'existe pas
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id            BIGSERIAL PRIMARY KEY,
            name          VARCHAR(100)  NOT NULL,
            email         VARCHAR(255)  UNIQUE NOT NULL,
            password_hash TEXT          NOT NULL,
            role          VARCHAR(20)   NOT NULL DEFAULT 'user',
            created_at    TIMESTAMPTZ   DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await
    .expect("Impossible de créer la table users");

    println!("[user-service] Table users prête ✓");

    let state = Arc::new(AppState { pool, jwt_secret });

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .layer(Extension(state))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    println!("[user-service] Démarré sur {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
