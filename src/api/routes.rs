use axum::Router;
use crate::{api::state::AppState, db::end_session};
use crate::api::handlers::health::health_check;
use tower_http::trace::TraceLayer;
use crate::api::handlers::session::{create_session, get_current_session, end_current_session};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/session", axum::routing::post(create_session))
        .route("/api/session/current", axum::routing::get(get_current_session))
        .route("/api/session/end", axum::routing::post(end_current_session))
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api, db::tests::setup_test_db};
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
