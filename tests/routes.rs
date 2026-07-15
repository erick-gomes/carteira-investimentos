use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Method, Request, StatusCode, header};
use axum::response::Response;
use carteira_investimentos::{AppState, routes::create_router};
use dotenvy::dotenv;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::env;
use tower::{Service, ServiceExt};

fn unique_string() -> String {
    format!(
        "uid_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

async fn send_request(
    app: &Router<()>,
    method: &Method,
    uri: &str,
    body: Option<String>,
    bearer: Option<&str>,
    cookie: Option<&str>,
) -> Response<Body> {
    let mut builder = Request::builder().method(method.clone()).uri(uri);

    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };

    let mut router = app.clone();
    let mut service = router.as_service::<Body>();
    service.ready().await.unwrap().call(request).await.unwrap()
}

fn app_state() -> AppState {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://erick:investimento@localhost:5432/carteira_investimentos".to_string()
    });
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "mysecretkey123".to_string());
    let jwt_secret = if jwt_secret.len() < 12 {
        "mysecretkey123".to_string()
    } else {
        jwt_secret
    };

    AppState {
        pool: PgPool::connect_lazy(&database_url).unwrap(),
        jwt_secret,
    }
}

async fn parse_json(response: Response<Body>) -> JsonValue {
    let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_route_returns_healthy() {
    let app = create_router().with_state(app_state());
    let response = send_request(&app, &Method::GET, "/api/v1/health", None, None, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body_bytes, "healthy");
}

#[tokio::test]
async fn public_and_protected_routes_are_registered() {
    let app = create_router().with_state(app_state());

    let scenarios = [
        (Method::GET, "/api/v1/health", StatusCode::OK),
        (Method::POST, "/api/v1/users", StatusCode::BAD_REQUEST),
        (Method::POST, "/api/v1/auth/login", StatusCode::BAD_REQUEST),
        (
            Method::POST,
            "/api/v1/auth/refresh",
            StatusCode::UNAUTHORIZED,
        ),
        (Method::GET, "/api/v1/user/me", StatusCode::UNAUTHORIZED),
        (Method::GET, "/api/v1/assets", StatusCode::UNAUTHORIZED),
        (Method::POST, "/api/v1/assets", StatusCode::UNAUTHORIZED),
        (
            Method::PATCH,
            "/api/v1/assets/00000000-0000-0000-0000-000000000000",
            StatusCode::UNAUTHORIZED,
        ),
        (
            Method::DELETE,
            "/api/v1/assets/00000000-0000-0000-0000-000000000000",
            StatusCode::UNAUTHORIZED,
        ),
    ];

    for (method, uri, expected_status) in scenarios {
        let response = send_request(&app, &method, uri, None, None, None).await;
        assert_eq!(
            response.status(),
            expected_status,
            "{} {} returned unexpected status",
            method,
            uri
        );
    }
}

#[tokio::test]
async fn full_user_and_asset_flow_works() {
    let app = create_router().with_state(app_state());
    let username = unique_string();
    let email = format!("{}@example.com", unique_string());
    let password = "senha123";

    let user_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
    })
    .to_string();

    let response = send_request(
        &app,
        &Method::POST,
        "/api/v1/users",
        Some(user_body),
        None,
        None,
    )
    .await;
    let status = response.status();
    if status != StatusCode::CREATED {
        let body_bytes = to_bytes(response.into_body(), 65536).await.unwrap();
        panic!(
            "create_user failed: {} {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    let user_json = parse_json(response).await;
    let user_id = user_json["id"].as_str().unwrap().to_string();
    assert_eq!(user_json["username"], username);
    assert_eq!(user_json["email"], email);

    let login_body = serde_json::json!({
        "username": username,
        "password": password,
    })
    .to_string();

    let response = send_request(
        &app,
        &Method::POST,
        "/api/v1/auth/login",
        Some(login_body),
        None,
        None,
    )
    .await;
    let status = response.status();
    if status != StatusCode::OK {
        let body_bytes = to_bytes(response.into_body(), 65536).await.unwrap();
        panic!(
            "login failed: {} {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    let cookie_header = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let refresh_cookie = cookie_header.split(';').next().unwrap().to_string();
    let access_token = parse_json(response).await["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let response = send_request(
        &app,
        &Method::GET,
        "/api/v1/user/me",
        None,
        Some(&access_token),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let me_json = parse_json(response).await;
    assert_eq!(me_json["id"], user_id);
    assert_eq!(me_json["email"], email);
    assert_eq!(me_json["username"], username);

    let refresh_response = send_request(
        &app,
        &Method::POST,
        "/api/v1/auth/refresh",
        None,
        None,
        Some(&refresh_cookie),
    )
    .await;
    let status = refresh_response.status();
    if status != StatusCode::OK {
        let body_bytes = to_bytes(refresh_response.into_body(), 65536).await.unwrap();
        panic!(
            "refresh_access_token failed: {} {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    let new_access_token = parse_json(refresh_response).await["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!new_access_token.is_empty());

    let asset_body = serde_json::json!({
        "name": "Ativo Teste",
        "category": "Renda Variável",
        "currency": "BRL",
        "ticker": "TEST",
        "quantity_raw": 150,
        "average_price_cents": 12345,
        "last_acquisition_date": "2026-07-15",
    })
    .to_string();

    let response = send_request(
        &app,
        &Method::POST,
        "/api/v1/assets",
        Some(asset_body),
        Some(&new_access_token),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let asset_json = parse_json(response).await;
    let asset_id = asset_json["id"].as_str().unwrap().to_string();
    assert_eq!(asset_json["name"], "Ativo Teste");
    assert_eq!(asset_json["ticker"], "TEST");

    let response = send_request(
        &app,
        &Method::GET,
        "/api/v1/assets",
        None,
        Some(&new_access_token),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let assets_json = parse_json(response).await;
    assert!(assets_json.is_array());
    assert!(
        assets_json
            .as_array()
            .unwrap()
            .iter()
            .any(|asset| asset["id"] == asset_id)
    );

    let patch_body = serde_json::json!({
        "name": "Ativo Teste Atualizado",
        "category": "Renda Fixa",
    })
    .to_string();

    let response = send_request(
        &app,
        &Method::PATCH,
        &format!("/api/v1/assets/{}", asset_id),
        Some(patch_body),
        Some(&new_access_token),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let patched_asset_json = parse_json(response).await;
    assert_eq!(patched_asset_json["name"], "Ativo Teste Atualizado");
    assert_eq!(patched_asset_json["category"], "Renda Fixa");

    let response = send_request(
        &app,
        &Method::DELETE,
        &format!("/api/v1/assets/{}", asset_id),
        None,
        Some(&new_access_token),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let deleted_asset_json = parse_json(response).await;
    assert_eq!(deleted_asset_json["id"], asset_id);
}
