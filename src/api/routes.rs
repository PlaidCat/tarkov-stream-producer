use axum::Router;
use crate::api::state::AppState;
use crate::api::handlers::health::health_check;
use tower_http::trace::TraceLayer;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::setup_test_db;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_api_router_has_health() {
        let pool = setup_test_db().await.expect("setup db");
        let app = super::api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
