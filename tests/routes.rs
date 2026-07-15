use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use carteira_investimentos::{AppState, routes::create_router};
use sqlx::PgPool;
use tower::util::ServiceExt;

#[tokio::test]
async fn health_route_returns_healthy() {
    let app_state = AppState {
        pool: PgPool::connect_lazy("postgres://postgres:postgres@localhost/test").unwrap(),
        jwt_secret: "secret".to_string(),
    };

    let app = create_router().with_state(app_state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body_bytes, "healthy");
}

#[tokio::test]
async fn public_and_protected_routes_are_registered() {
    let app_state = AppState {
        pool: PgPool::connect_lazy("postgres://postgres:postgres@localhost/test").unwrap(),
        jwt_secret: "secret".to_string(),
    };

    let app = create_router().with_state(app_state);

    let scenarios = [
        ("GET", "/api/v1/health", StatusCode::OK),
        ("POST", "/api/v1/users", StatusCode::BAD_REQUEST),
        ("POST", "/api/v1/auth/login", StatusCode::BAD_REQUEST),
        ("POST", "/api/v1/auth/refresh", StatusCode::UNAUTHORIZED),
        ("GET", "/api/v1/user/me", StatusCode::UNAUTHORIZED),
        ("GET", "/api/v1/assets", StatusCode::UNAUTHORIZED),
        ("POST", "/api/v1/assets", StatusCode::UNAUTHORIZED),
        (
            "PATCH",
            "/api/v1/assets/00000000-0000-0000-0000-000000000000",
            StatusCode::UNAUTHORIZED,
        ),
        (
            "DELETE",
            "/api/v1/assets/00000000-0000-0000-0000-000000000000",
            StatusCode::UNAUTHORIZED,
        ),
    ];

    for (method, uri, expected_status) in scenarios {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            expected_status,
            "{} {} returned unexpected status",
            method,
            uri
        );
    }
}
