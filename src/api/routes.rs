use axum::Router;
use crate::api::state::AppState;
use tower_http::trace::TraceLayer;
use crate::api::handlers::health::health_check;
use crate::api::handlers::session::{create_session, get_current_session, end_current_session};
use crate::api::handlers::raid::{create_raid, get_current_raid, transition_state, end_raid};
use crate::api::handlers::kill::{add_kill, get_kills, add_batch_kills};
use crate::api::handlers::stats::{get_current_session_stats, get_raid_stats};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/session", axum::routing::post(create_session))
        .route("/api/session/current", axum::routing::get(get_current_session))
        .route("/api/session/end", axum::routing::post(end_current_session))
        .route("/api/raid", axum::routing::post(create_raid))
        .route("/api/raid/current", axum::routing::get(get_current_raid))
        .route("/api/raid/transition", axum::routing::post(transition_state))
        .route("/api/raid/end", axum::routing::post(end_raid))
        .route("/api/raid/{raid_id}/kills", axum::routing::post(add_kill).get(get_kills))
        .route("/api/raid/current/kills/batch", axum::routing::post(add_batch_kills))
        .route("/api/stats/session/current", axum::routing::get(get_current_session_stats))
        .route("/api/stats/raid/{raid_id}", axum::routing::get(get_raid_stats))
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
