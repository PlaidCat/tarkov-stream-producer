use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use time::{OffsetDateTime, Duration};
use crate::db::*;

use crate::models::*;
use std::collections::HashMap;

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
    let transitions = get_raid_transitions(&pool, raid_id).await?;

    let mut state_durations: HashMap<String, Duration> = HashMap::new();

    for i in 0..transitions.len() - 1 {
        let current = &transitions[i];
        let next = &transitions[i + 1];

        let duration = next.transitioned_at - current.transitioned_at;

        //This is confusing to me but is more efficient than how I would have thought to do this
        // if let Some(existing) = state_durations.get_mut(current.to_state.clone()) {
        //      *existing = *existing + duration; // update if exists
        // } else {
        //      state_durations.insert(key, duration); // Insert if New
        // }
        // I've seen the below and had Claude explain it to me, this is a note to explain this
        // pattern to me
        state_durations.entry(current.to_state.clone())
            .and_modify(|d| *d = *d + duration)
            .or_insert(duration);
    }

    // Move Ownership from HashMap into an iter of Iterator<Key, value> tuples
    // Map these tuples into a new StateTime struct, and then collect
    // the mapped StateTime structs into the Vec<>
    let state_times: Vec<StateTime> = state_durations.into_iter()
            .map(|(state, duration)| StateTime { state, duration })
            .collect();

    Ok(state_times)
}

#[cfg(test)]
mod tests {
    use crate::db::tests::setup_test_db;

    use super::*;

    #[tokio::test]
    async fn test_calculate_time_before_first_raid() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;

        let zero_session = calculate_time_before_first_raid(&pool).await?;
        assert_eq!(zero_session.duration, Duration::ZERO);


        let base_time = OffsetDateTime::now_utc();

        //Session 1: Started at base time first raid is 10 min in
        let session1_start = base_time;
        let session1 = create_session(
            &pool, SessionType::Stream, Some("Session 1".into()), Some(session1_start)).await?;

        let raid1_start = session1_start + time::Duration::minutes(10);
        let _raid1 = create_raid(&pool, session1, "customs", CharacterType::PMC, GameMode::PVE, Some(raid1_start)).await?;

        //Session2 started 1 day later, first raid 30 min after
        let session2_start = base_time + time::Duration::days(1);
        let session2 = create_session(
            &pool, SessionType::Stream, Some("Session 2".into()), Some(session2_start)).await?;

        let raid2_start = session2_start + time::Duration::minutes(30);
        let _raid2 = create_raid(
            &pool, session2, "Streets of Tarkov:", CharacterType::PMC, GameMode::PVE, Some(raid2_start)).await?;

        //Session3 started 2 days later, first raid 5 min after
        let session3_start = session2_start + time::Duration::days(2);
        let session3 = create_session(
            &pool, SessionType::Stream, Some("Session 3".into()), Some(session3_start)).await?;

        let raid3_start = session3_start + time::Duration::minutes(5);
        let _raid3 = create_raid(
            &pool, session3, "Shoreline", CharacterType::PMC, GameMode::PVE, Some(raid3_start)).await?;

        // Calculate the dead time
        let result = calculate_time_before_first_raid(&pool).await?;

        assert_eq!(result.sessions, 3, "should have analyzed 3 sessions");

        assert_eq!(result.duration, time::Duration::minutes(15), "Average should be 15 minutes");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_time_single_state_visit() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;

        // Base time for all calculations
        let base_time = OffsetDateTime::now_utc();

        // Start Session at base_time
        let session_start = base_time;
        let session_id = create_session(
            &pool,
            SessionType::Stream,
            Some("Test Stream".into()),
            Some(session_start)
        ).await?;

        // Wait 20 min before starting first raid (session overhead time)
        // This gap is tracked by calculate_time_before_first_raid()
        let raid_start = session_start + time::Duration::minutes(20);

        // Start first raid on Shoreline for PVE
        // Raid begins in pre_raid_setup state
        let raid_id = create_raid(
            &pool,
            session_id,
            "Shoreline",
            CharacterType::PMC,
            GameMode::PVE,
            Some(raid_start)
        ).await?;

        // State flow with timestamps
        let mut time = raid_start;

        // Initial state: pre_raid_setup (selecting map, insurance)
        log_state_transition(&pool, raid_id, "pre_raid_setup", Some(time)).await?;

        // Spend 3 min in pre-raid setup
        time = time + time::Duration::minutes(3);

        // Enter queue - wait 5 min
        log_state_transition(&pool, raid_id, "queuing", Some(time)).await?;
        time = time + time::Duration::minutes(5);

        // Matched and deploying (loading screens) - 2 min
        log_state_transition(&pool, raid_id, "deploying_committed", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        // Raid active for 17 min
        log_state_transition(&pool, raid_id, "raid_active", Some(time)).await?;

        // Add 3 kills (all scavs) during the raid
        let kill_time_1 = time + time::Duration::minutes(5);
        add_kill(&pool, raid_id, "scav", Some("AK-74M".to_string()), Some(false), Some(kill_time_1)).await?;

        let kill_time_2 = time + time::Duration::minutes(10);
        add_kill(&pool, raid_id, "scav", Some("Mosin".to_string()), Some(true), Some(kill_time_2)).await?;

        let kill_time_3 = time + time::Duration::minutes(15);
        add_kill(&pool, raid_id, "scav", Some("SKS".to_string()), Some(false), Some(kill_time_3)).await?;

        time = time + time::Duration::minutes(17);

        // Extract - raid ending (1 min)
        log_state_transition(&pool, raid_id, "raid_ending", Some(time)).await?;
        time = time + time::Duration::minutes(1);

        // Post-raid review (statistics, experience) - 2 min
        log_state_transition(&pool, raid_id, "post_raid_review", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        // Final state: survived
        log_state_transition(&pool, raid_id, "survived", Some(time)).await?;
        time = time + time::Duration::seconds(30);

        // End the raid
        end_raid(&pool, raid_id, None, Some("Tunnel".into())).await?;

        // Calculate time spent in each state
        let states = calculate_time_in_state(&pool, raid_id).await?;

        // Assert all expected durations
        let pre_raid = states.iter()
            .find(|s| s.state == "pre_raid_setup")
            .expect("Should have pre_raid_setup state");
        assert_eq!(pre_raid.duration, time::Duration::minutes(3));

        let queuing = states.iter()
            .find(|s| s.state == "queuing")
            .expect("Should have queuing state");
        assert_eq!(queuing.duration, time::Duration::minutes(5));

        let deploying = states.iter()
            .find(|s| s.state == "deploying_committed")
            .expect("Should have deploying_committed state");
        assert_eq!(deploying.duration, time::Duration::minutes(2));

        let raid_active = states.iter()
            .find(|s| s.state == "raid_active")
            .expect("Should have raid_active state");
        assert_eq!(raid_active.duration, time::Duration::minutes(17));

        let raid_ending = states.iter()
            .find(|s| s.state == "raid_ending")
            .expect("Should have raid_ending state");
        assert_eq!(raid_ending.duration, time::Duration::minutes(1));

        let post_raid = states.iter()
            .find(|s| s.state == "post_raid_review")
            .expect("Should have post_raid_review state");
        assert_eq!(post_raid.duration, time::Duration::minutes(2));

        let survived = states.iter()
            .find(|s| s.state == "survived")
            .expect("Should have survived state");
        assert_eq!(survived.duration, time::Duration::seconds(30));

        pool.close().await;
        Ok(())
    }
}
