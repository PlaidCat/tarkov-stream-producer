use axum::{extract::State, response::Html};
use askama::Template;
use crate::api::{error::AppError, state::AppState};
use crate::db;
use crate::stats::{calculate_session_stats, SessionStats};
use crate::models::{StreamSession, Raid, CharacterType, GameMode};
use time::{OffsetDateTime, Duration};

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.whole_seconds().abs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub active_session: Option<StreamSession>,
    pub formatted_session_duration: String,
    pub active_raid: Option<Raid>,
    pub session_stats: Option<SessionStats>,
    pub formatted_avg_duration: String,
    pub formatted_avg_queue_time: String,
    pub formatted_total_raid_time: String,
    pub formatted_total_queue_time: String,
    pub formatted_total_stash_time: String,
    pub recent_raids: Vec<RaidViewModel>,
}

pub struct RaidViewModel {
    pub raid_id: i64,
    pub map_name: String,
    pub character_type: CharacterType,
    pub game_mode: GameMode,
    pub current_state: String,
    pub duration: String,
    pub ended_at: Option<OffsetDateTime>,
}

pub async fn index(
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let active_session = db::get_active_session(&state.pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let active_raid = db::get_active_raid(&state.pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let mut session_stats = None;
    let mut formatted_avg_duration = "00:00".to_string();
    let mut formatted_avg_queue_time = "00:00".to_string();
    let mut formatted_total_raid_time = "00:00".to_string();
    let mut formatted_total_queue_time = "00:00".to_string();
    let mut formatted_total_stash_time = "00:00".to_string();
    let mut formatted_session_duration = "00:00".to_string();
    let mut recent_raids = Vec::new();

    if let Some(ref session) = active_session {
        formatted_session_duration = format_duration(OffsetDateTime::now_utc() - session.started_at);
        
        let stats = calculate_session_stats(&state.pool, session.session_id)
            .await
            .map_err(AppError::DatabaseError)?;
        
        formatted_avg_duration = format_duration(stats.avg_raid_duration);
        formatted_avg_queue_time = format_duration(stats.time_breakdown.avg_queue_time);
        formatted_total_raid_time = format_duration(stats.time_breakdown.total_raid_time);
        formatted_total_queue_time = format_duration(stats.time_breakdown.total_queue_time);
        formatted_total_stash_time = format_duration(stats.time_breakdown.total_stash_time);
        session_stats = Some(stats);
        
        let raids = db::get_raids_for_session(&state.pool, session.session_id)
            .await
            .map_err(AppError::DatabaseError)?;
        
        // Convert to ViewModel
        recent_raids = raids.into_iter().rev().take(5).map(|r| {
            let duration_str = if let Some(ended) = r.ended_at {
                format_duration(ended - r.started_at)
            } else {
                "Active".to_string()
            };

            RaidViewModel {
                raid_id: r.raid_id,
                map_name: r.map_name,
                character_type: r.character_type,
                game_mode: r.game_mode,
                current_state: r.current_state,
                duration: duration_str,
                ended_at: r.ended_at,
            }
        }).collect();
    }

    let rendered = IndexTemplate {
        active_session,
        formatted_session_duration,
        active_raid,
        session_stats,
        formatted_avg_duration,
        formatted_avg_queue_time,
        formatted_total_raid_time,
        formatted_total_queue_time,
        formatted_total_stash_time,
        recent_raids,
    }
    .render()
    .map_err(AppError::TemplateError)?;

    Ok(Html(rendered))
}

#[derive(Template)]
#[template(path = "sessions.html")]
pub struct SessionsTemplate {
    pub sessions: Vec<SessionViewModel>,
}

pub struct SessionViewModel {
    pub session_id: i64,
    pub session_type: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration: String,
    pub notes: Option<String>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let sessions = db::get_all_sessions(&state.pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let view_models = sessions.into_iter().rev().map(|s| {
        let duration = if let Some(ended) = s.ended_at {
            format_duration(ended - s.started_at)
        } else {
            "Active".to_string()
        };

        SessionViewModel {
            session_id: s.session_id,
            session_type: s.session_type.map(|t| t.to_string()),
            started_at: s.started_at.to_string(),
            ended_at: s.ended_at.map(|e| e.to_string()),
            duration,
            notes: s.notes,
        }
    }).collect();

    let rendered = SessionsTemplate { sessions: view_models }
        .render()
        .map_err(AppError::TemplateError)?;

    Ok(Html(rendered))
}

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub global_stats: SessionStats,
    pub formatted_avg_duration: String,
    pub formatted_avg_stash_time: String,
    pub formatted_avg_queue_time: String,
    pub formatted_total_raid_time: String,
    pub formatted_total_queue_time: String,
    pub formatted_total_stash_time: String,
}

pub async fn global_stats(
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let global_stats = crate::stats::calculate_global_stats(&state.pool, None)
        .await
        .map_err(AppError::DatabaseError)?;

    let gap_stats = crate::stats::calculate_time_between_raids_global(&state.pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let formatted_avg_duration = format_duration(global_stats.avg_raid_duration);
    let formatted_avg_stash_time = format_duration(gap_stats.avg_gap);
    let formatted_avg_queue_time = format_duration(global_stats.time_breakdown.avg_queue_time);
    let formatted_total_raid_time = format_duration(global_stats.time_breakdown.total_raid_time);
    let formatted_total_queue_time = format_duration(global_stats.time_breakdown.total_queue_time);
    let formatted_total_stash_time = format_duration(global_stats.time_breakdown.total_stash_time);

    let rendered = StatsTemplate { 
        global_stats,
        formatted_avg_duration,
        formatted_avg_stash_time,
        formatted_avg_queue_time,
        formatted_total_raid_time,
        formatted_total_queue_time,
        formatted_total_stash_time,
    }
        .render()
        .map_err(AppError::TemplateError)?;

    Ok(Html(rendered))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::setup_test_db;
    use crate::models::SessionType;
    use axum::{body::Body, http::Request, Router};
    use tower::ServiceExt; // for oneshot

    fn app(pool: sqlx::SqlitePool) -> Router {
        crate::api::routes::api_router().with_state(AppState::new(pool))
    }

    #[tokio::test]
    async fn test_index_renders() {
        let pool = setup_test_db().await.expect("setup db");
        let response = app(pool)
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
        // HTML content verification is tricky without parsing, but 200 means template rendered
    }

    #[tokio::test]
    async fn test_sessions_renders() {
        let pool = setup_test_db().await.expect("setup db");
        db::create_session(&pool, SessionType::Stream, None, None).await.unwrap();
        
        let response = app(pool)
            .oneshot(Request::get("/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_stats_renders() {
        let pool = setup_test_db().await.expect("setup db");
        let response = app(pool)
            .oneshot(Request::get("/stats").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_format_duration_logic() {
        assert_eq!(format_duration(Duration::seconds(45)), "00:45");
        assert_eq!(format_duration(Duration::seconds(61)), "01:01");
        assert_eq!(format_duration(Duration::minutes(59) + Duration::seconds(59)), "59:59");
        assert_eq!(format_duration(Duration::hours(1) + Duration::seconds(1)), "01:00:01");
        assert_eq!(format_duration(Duration::ZERO), "00:00");
    }

    #[tokio::test]
    async fn test_index_empty_state() {
        let pool = setup_test_db().await.expect("setup db");
        let response = app(pool)
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        
        assert!(body_str.contains("No Active Session"));
        assert!(body_str.contains("Start New Session"));
    }

    #[tokio::test]
    async fn test_index_with_active_session() {
        let pool = setup_test_db().await.expect("setup db");
        db::create_session(&pool, SessionType::Stream, Some("Vibing in Tarkov".into()), None).await.unwrap();
        
        let response = app(pool)
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        
        assert!(body_str.contains("Vibing in Tarkov"));
        assert!(body_str.contains("End Session"));
    }
}
