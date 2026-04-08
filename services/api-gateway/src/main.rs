use axum::{
    body::Body,
    extract::Extension,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use bytes::Bytes;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;

// ── Claims (miroir du user-service) ──────────────────────────────────────────

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
    client: reqwest::Client,
    jwt_secret: String,
    user_service_url: String,
    cotation_service_url: String,
    scraper_service_url: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
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

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "api-gateway",
        "routes": [
            "POST /api/auth/register",
            "POST /api/auth/login",
            "GET  /api/auth/me          [JWT]",
            "POST /api/cotation         [JWT]",
            "GET  /api/cotation/history [JWT]",
            "GET  /api/listings         [JWT]",
            "POST /api/scrape/sync      [JWT]",
        ]
    }))
}

async fn proxy(
    Extension(state): Extension<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = uri.path();

    // ── Routing : déterminer le service cible et si auth est requise ──────────
    let (target_base, requires_auth): (String, bool) =
        if path.starts_with("/api/auth") {
            (state.user_service_url.clone(), false)
        } else if path.starts_with("/api/cotation") {
            (state.cotation_service_url.clone(), true)
        } else if path.starts_with("/api/listings")
            || path.starts_with("/api/scrape")
        {
            (state.scraper_service_url.clone(), true)
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Route inconnue", "path": path})),
            )
                .into_response();
        };

    // ── Validation JWT pour les routes protégées ──────────────────────────────
    let mut user_headers: Vec<(String, String)> = Vec::new();

    if requires_auth {
        match extract_bearer(&headers) {
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Token JWT requis",
                        "hint": "Header: Authorization: Bearer <token>"
                    })),
                )
                    .into_response();
            }
            Some(token) => match validate_jwt(&token, &state.jwt_secret) {
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Token invalide ou expiré"})),
                    )
                        .into_response();
                }
                Ok(claims) => {
                    user_headers.push(("X-User-Id".into(), claims.sub.to_string()));
                    user_headers.push(("X-User-Email".into(), claims.email));
                    user_headers.push(("X-User-Role".into(), claims.role));
                }
            },
        }
    }

    // ── Construire l'URL cible ────────────────────────────────────────────────
    // /api/cotation → /cotation  |  /api/auth/login → /auth/login
    let downstream_path = path.strip_prefix("/api").unwrap_or(path);
    let query = uri
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!("{}{}{}", target_base, downstream_path, query);

    println!("[Gateway] {} {} → {}", method, path, target_url);

    // ── Construire et envoyer la requête downstream ───────────────────────────
    let req_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);

    let mut req_builder = state.client.request(req_method, &target_url).body(body.to_vec());

    // Copier les headers (sans host ni content-length)
    for (name, value) in headers.iter() {
        let n = name.as_str();
        if n != "host" && n != "content-length" {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(n, v);
            }
        }
    }

    // Ajouter les headers utilisateur extraits du JWT
    for (k, v) in user_headers {
        req_builder = req_builder.header(k, v);
    }

    match req_builder.send().await {
        Err(e) => {
            eprintln!("[Gateway] Erreur proxy vers {}: {}", target_url, e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "Service en aval indisponible", "detail": e.to_string()})),
            )
                .into_response()
        }
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let mut response_builder = Response::builder().status(status);

            // Copier les headers de la réponse (content-type, etc.)
            for (name, value) in resp.headers().iter() {
                if name.as_str() != "transfer-encoding" {
                    response_builder = response_builder.header(name, value);
                }
            }

            let body_bytes = resp.bytes().await.unwrap_or_default();

            response_builder
                .body(Body::from(body_bytes))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Erreur construction réponse")
                        .into_response()
                })
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let jwt_secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_in_prod".to_string());
    let user_service_url =
        env::var("USER_SERVICE_URL").unwrap_or_else(|_| "http://user-service:8000".to_string());
    let cotation_service_url = env::var("COTATION_SERVICE_URL")
        .unwrap_or_else(|_| "http://cotation-service:8000".to_string());
    let scraper_service_url = env::var("SCRAPER_SERVICE_URL")
        .unwrap_or_else(|_| "http://scraper-service:8000".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Impossible de créer le client HTTP");

    let state = Arc::new(AppState {
        client,
        jwt_secret,
        user_service_url,
        cotation_service_url,
        scraper_service_url,
    });

    let app = Router::new()
        .route("/health", get(health))
        .fallback(proxy)
        .layer(Extension(state))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    println!("[api-gateway] Démarré sur {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
