use axum::{extract::State, Json};
use serde::Serialize;
use crate::api::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::state::AppState;
    use crate::db::tests::setup_test_db;
    use axum::{body::Body, http::Request, Router};
    use tower::ServiceExt;
    use http::StatusCode;

    fn health_router(state: AppState) -> Router {
        Router::new()
            .route("/health", axum::routing::get(health_check))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_returns_ok() {
        let pool = setup_test_db().await.expect("setup db");
        let app = health_router(AppState::new(pool));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_returns_connected() {
        let pool = setup_test_db().await.expect("setup db");
        let app = health_router(AppState::new(pool));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
        assert_eq!(json["database"], "connected");
    }
}
