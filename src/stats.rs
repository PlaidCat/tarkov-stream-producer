use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use time::{OffsetDateTime, Duration};
use crate::db::*;

use std::{collections::HashMap};

use crate::models::*;

#[derive(Debug, Clone)]
pub struct StateTime {
    pub state: String,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct FirstRaidDelay {
    pub duration: Duration,
    pub sessions: i64,
    pub last_session: Duration,
}

pub async fn calculate_time_before_first_raid(
    pool: &SqlitePool
) -> Result<FirstRaidDelay, sqlx::Error> {
    let sessions = get_all_sessions(pool).await?;
    let mut total_delay = Duration::ZERO;
    let mut total_sessions = 0i64;
    let mut last_session_duration = Duration::ZERO;

    for session in sessions.iter() {
        if let Some(first_raid) = get_first_raid_for_session(pool, session.session_id).await? {
            let delay = first_raid.started_at - session.started_at;
            total_delay = total_delay + delay;
            total_sessions += 1;

            if total_sessions == 1 {
                last_session_duration = delay;
            }
        }

    }

    let avg_duration = if total_sessions > 0 {
        total_delay / total_sessions as i32
    } else {
        Duration::ZERO
    };

    Ok(FirstRaidDelay { 
        duration: avg_duration,
        sessions: total_sessions,
        last_session: last_session_duration
    })

}

pub async fn calculate_time_in_state(
    pool: &SqlitePool,
    raid_id: i64
) -> Result<Vec<StateTime>, sqlx::Error> {
    //TODO Implement actual calculation logic
    let transitions = get_raid_transitions(&pool, raid_id).await?;
    let raid = get_active_raid(&pool).await?.expect("Expected Active Raid");

    for i in 0..transitions.len() {
        println!("{:?}", transitions[i])
    }

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
//    use std::intrinsics::mir::Offset;

    use crate::db::tests::setup_test_db;

    use super::*;

    #[tokio::test]
    async fn test_calculate_time_single_state_visit() -> Result<(), sqlx::Error> {
        let pool = crate::db::tests::setup_test_db().await.expect("Failed to setup test db");
        let session_id = create_session(&pool, SessionType::Stream, Some("Test Stream".into())).await?;
        let session_start = OffsetDateTime::now_utc();
        // Session start had 20min dicking around in the Stash / Menus


        //This is start of stream raid set up the 
        let raid_id = create_raid(&pool, session_id, "customs", CharacterType::PMC, GameMode::PVE).await?;
        let mut time = OffsetDateTime::now_utc();
        log_state_transition(&pool, raid_id, "stash_management", Some(time)).await.expect("Set Initial Raid State");
        time = time + time::Duration::seconds(120);

        log_state_transition(&pool, raid_id, "queue", Some(time)).await.expect("Expected State Transiton to queue");
        time = time + time::Duration::seconds(60);
        log_state_transition(&pool, raid_id,  "in_raid", Some(time)).await.expect("Expected State Transition to in_raid");
        time = time + time::Duration::seconds(1200);
        log_state_transition(&pool, raid_id, "stash_management", Some(time)).await.expect("Expected State Transitino to Stash");

        let states = calculate_time_in_state(&pool, raid_id).await.expect("sates");

        Ok(())
    }
}
