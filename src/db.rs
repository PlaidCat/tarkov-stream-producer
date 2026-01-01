use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Error, Transaction};
use time::OffsetDateTime;

use crate::models::{CharacterType, Raid, GameMode, SessionType, StreamSession, RaidStateTransition};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, Error> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

// Migrations are the Database as code process of iniiing / changing the schemea
// this is new to the author 
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}

// ================================================================================================
// Session Operations
// ================================================================================================

pub async fn create_session(pool: &SqlitePool, session_type: SessionType, notes: Option<String>,) -> Result<i64, Error> {
    let id = sqlx::query!(r#"INSERT INTO stream_sessions ( session_type, notes)
        VALUES (?, ?)
        RETURNING session_id"#,
        session_type,
        notes)
        .fetch_one(pool)
        .await?
        .session_id;

    Ok(id)
}

pub async fn end_session(pool: &SqlitePool, session_id: i64) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE stream_sessions
        SET ended_at = CURRENT_TIMESTAMP
        WHERE session_id = ?
        "#,
        session_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_active_session(pool: &SqlitePool) -> Result<Option<StreamSession>, Error> {
    sqlx::query_as!(
        StreamSession,
        r#"
        SELECT
            session_id as "session_id!",
            started_at,
            ended_at,
            session_type AS "session_type: SessionType",
            notes
        FROM stream_sessions
        WHERE ended_at IS NULL
        ORDER BY started_at DESC
        LIMIT 1
        "#
    ).fetch_optional(pool).await
}

// ================================================================================================
// Raid Operations
// ================================================================================================
pub async fn create_raid(pool: &SqlitePool, session_id: i64, map_name: &str, 
    character_type: CharacterType, game_mode: GameMode,) -> Result<i64, Error> {
    let id = sqlx::query!(
        r#"
        INSERT INTO raids (session_id, map_name, character_type, game_mode)
        VALUES (?, ?, ?, ?)
        RETURNING raid_id
        "#,
        session_id,
        map_name,
        character_type,
        game_mode
    )
    .fetch_one(pool)
    .await?
    .raid_id;

    Ok(id)
}

pub async fn end_raid(pool: &SqlitePool, raid_id: i64, end_time: Option<OffsetDateTime>, 
                      extract_location: Option<String>,) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE raids
        SET ended_at = COALESCE(?, CURRENT_TIMESTAMP), extract_location = ?
        WHERE raid_id = ?
        "#,
        end_time,
        extract_location,
        raid_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_active_raid(pool: &SqlitePool) -> Result<Option<Raid>, Error> {
    sqlx::query_as!(
        Raid,
        r#"
        SELECT
            raid_id as "raid_id!",
            session_id as "session_id!",
            started_at,
            ended_at,
            map_name as "map_name!",
            character_type AS "character_type: CharacterType",
            game_mode AS "game_mode: GameMode",
            current_state as "current_state!",
            extract_location
        FROM raids
        WHERE ended_at IS NULL
        ORDER BY started_at DESC
        LIMIT 1
        "#
    ).fetch_optional(pool).await
}

// ================================================================================================
// State Transition Operations
// ================================================================================================
pub async fn log_state_transition(
    pool: &SqlitePool,
    raid_id: i64,
    to_state: &str,
    timestamp: Option<OffsetDateTime>,
) -> Result<(), Error> {
    let mut tx: Transaction<'_, sqlx::Sqlite> = pool.begin().await?;

    let from_state: String = sqlx::query_scalar!(
        "SELECT current_state FROM raids WHERE raid_id = ?",
        raid_id
    )
    .fetch_one(&mut *tx)
    .await?;

    let ts = timestamp.unwrap_or_else(|| OffsetDateTime::now_utc());

    sqlx::query!(
        r#"
        INSERT INTO raid_state_transitions (raid_id, from_state, to_state, transitioned_at)
        VALUES (?, ?, ?, ?)
        "#,
        raid_id,
        from_state,
        to_state,
        ts
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE raids SET current_state = ? WHERE raid_id = ?",
        to_state,
        raid_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_raid_transitions(
    pool: &SqlitePool, 
    raid_id: i64
) -> Result<Vec<RaidStateTransition>, Error> {
    sqlx::query_as!(
        RaidStateTransition,
        r#"
        SELECT
            transition_id as "transition_id!",
            raid_id as "raid_id!",
            from_state,
            to_state as "to_state!",
            transitioned_at
        FROM raid_state_transitions
        WHERE raid_id = ?
        ORDER BY transitioned_at ASC
        "#,
        raid_id
    )
    .fetch_all(pool)
    .await
}
#[cfg(test)]
mod tests {
    use tokio::time::sleep;
    use std::time::Duration;

    use super::*;

    async fn setup_test_db() -> Result<SqlitePool, Error> {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("Failed to create pool");
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");
        Ok(pool)
    }

    #[tokio::test]
    async fn test_init_schema() {
        let pool = setup_test_db().await.expect("Failed to setup_test_db");

        let table_exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='stream_sessions'"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check table exists");

        assert_eq!(table_exists.0, 1);
    }

    #[tokio::test]
    async fn test_get_transitions_for_nonexistent_raid() -> Result<(), Error> {
        let pool = setup_test_db().await?;

        let transition = get_raid_transitions(&pool, 999).await?;
        assert_eq!(transition.len(), 0); //should return empty vec, not error

        Ok(())
    }

    #[tokio::test]
    async fn test_session_lifecycle() -> Result<(), Error> {
        let pool = setup_test_db().await.expect("Failed to setup_test_db");


        //start Session
        let session_id = create_session(&pool, SessionType::Stream, Some("Test Stream".into())).await?;
        assert_eq!(session_id, 1);

        // Get Active Session
        let active = get_active_session(&pool).await?.expect("There should be an active session");
        assert_eq!(active.session_id, session_id);
        assert_eq!(active.session_type, Some(SessionType::Stream));

        // Start Raid
        let raid_id = create_raid(&pool, session_id, "customs", CharacterType::PMC, GameMode::PVE).await?;
        assert_eq!(raid_id, 1);

        let active_raid = get_active_raid(&pool).await?.expect("Should have active raid");
        assert_eq!(active_raid.map_name, "customs");
        assert_eq!(active_raid.current_state, "stash_management");

        let mut time = OffsetDateTime::now_utc();

        // Transitions
        log_state_transition(&pool, raid_id, "queue", Some(time)).await.expect("Should have done a state transition");
        time = time + time::Duration::seconds(10);
        log_state_transition(&pool, raid_id, "in_raid", Some(time)).await.expect("Should have done another state transition");

        let transitions = get_raid_transitions(&pool, raid_id).await?;
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to_state, "queue");
        assert_eq!(transitions[1].from_state, Some("queue".into()));
        assert_eq!(transitions[1].to_state, "in_raid");

        // Validate that we waited 10seconds in queue
        let time_diff = transitions[1].transitioned_at - transitions[0].transitioned_at;
        assert_eq!(time_diff, time::Duration::seconds(10), 
            "Expected 10 seconds between transitions");

        // End Raid
        end_raid(&pool, raid_id, None, None).await.expect("Failed to end_raid()");

        //End Session
        end_session(&pool, session_id).await.expect("Failed to end_session()");

        let active = get_active_session(&pool).await?;
        assert!(active.is_none());

        pool.close().await;
        Ok(())
    }
}
