use sqlx::sqlite::{SqlitePool};
use time::{Duration};
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

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_raids: i64,
    pub survived_raids: i64,
    pub survival_rate: f64,
    pub total_kills: i64,
    pub kd_ratio: f64,
    pub avg_raid_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct SessionComparison {
    pub current: SessionStats,
    pub all_time: SessionStats,
}

#[derive(Debug, Clone)]
pub struct ModeStats {
    pub pve: SessionStats,
    pub pvp: SessionStats,
}

pub async fn compare_session_to_mode_global(
    pool: &SqlitePool,
    session_id: i64,
    game_mode_filter: Option<GameMode>,
) -> Result<SessionComparison, sqlx::Error> {
    let current = calculate_session_stats(pool, session_id).await?;
    let all_time = calculate_global_stats(pool, game_mode_filter).await?;

    Ok(SessionComparison { current, all_time })
}

pub async fn get_mode_stats_for_session (
    pool: &SqlitePool,
    session_id: i64,
) -> Result<ModeStats, sqlx::Error> {
    let raids = get_raids_for_session(pool, session_id).await?;

    let pve_raids: Vec<Raid> = raids.iter().filter(|r| r.game_mode == GameMode::PVE).cloned().collect();
    let pvp_raids: Vec<Raid> = raids.iter().filter(|r| r.game_mode == GameMode::PVP).cloned().collect();

    let pve = calculate_stats_from_raids(pool, pve_raids).await?;
    let pvp = calculate_stats_from_raids(pool, pvp_raids).await?;

    Ok(ModeStats { pve, pvp })
}

pub async fn calculate_session_stats(
    pool: &SqlitePool,
    session_id: i64
) -> Result<SessionStats, sqlx::Error> {
    let raids = get_raids_for_session(pool, session_id).await?;
    calculate_stats_from_raids(pool, raids).await
}

pub async fn calculate_global_stats(
    pool: &SqlitePool,
    game_mode_filter: Option<GameMode>
) -> Result<SessionStats, sqlx::Error> {
    let all_raids = get_all_raids(pool).await?;

    let filtered_raids: Vec<Raid> = if let Some(mode) = game_mode_filter {
        all_raids.into_iter().filter(|r| r.game_mode == mode).collect()
    } else {
        all_raids
    };

    calculate_stats_from_raids(pool, filtered_raids).await
}

async fn calculate_stats_from_raids(
    pool: &SqlitePool,
    raids: Vec<Raid>
) -> Result<SessionStats, sqlx::Error> {
    let total_raids = raids.len() as i64;
    let mut survived_raids = 0;
    let mut total_kills = 0;
    let mut total_duration = Duration::ZERO;
    let mut duration_count = 0;

    for raid in &raids {
        if raid.current_state == "survived" {
            survived_raids += 1;
        }

        if let Some(ended_at) = raid.ended_at {
            total_duration += ended_at - raid.started_at;
            duration_count += 1;
        }

        let kills = get_kills_for_raid(pool, raid.raid_id).await?;
        total_kills += kills.len() as i64;
    }

    let deaths = total_raids - survived_raids;

    let survival_rate = if total_raids > 0 {
        survived_raids as f64 / total_raids as f64
    } else {
        0.0
    };

    let kd_ratio = if deaths > 0 {
        total_kills as f64 / deaths as f64
    } else {
        total_kills as f64
    };

    let avg_duration = if duration_count > 0 {
        total_duration / duration_count as i32
    } else {
        Duration::ZERO
    };

    Ok(SessionStats {
        total_raids,
        survived_raids,
        survival_rate,
        total_kills,
        kd_ratio,
        avg_raid_duration: avg_duration,
    })
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
    use time::OffsetDateTime;

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

        assert!(states.iter().find(|s| s.state == "survived").is_none(), 
            "Terminal states should not have durations");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_session_stats() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();

        // Create Session
        let session_id = create_session(&pool, SessionType::Stream, Some("Stats Test".into()), Some(base_time)).await?;

        // Raid 1: Survived, 30 min, 2 kills
        let start1 = base_time + time::Duration::minutes(10);
        let raid1 = create_raid(&pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, Some(start1)).await?;

        // Add 2 Kills
        add_kill(&pool, raid1, "scav", None, None, Some(start1 + time::Duration::minutes(5))).await?;
        add_kill(&pool, raid1, "pmc", None, None, Some(start1 + time::Duration::minutes(10))).await?;

        // End as Survived
        log_state_transition(&pool, raid1, "survived", Some(start1 + time::Duration::minutes(30))).await?;
        end_raid(&pool, raid1, Some(start1 + time::Duration::minutes(30)), Some("Crossroads".into())).await?;

        // Raid 2: KIA (Died), 15 min, 1 kill
        let start2 = base_time + time::Duration::minutes(60);
        let raid2 = create_raid(&pool, session_id, "Factory", CharacterType::PMC, GameMode::PVP, Some(start2)).await?;

        // Add 1 Kill
        add_kill(&pool, raid2, "scav", None, None, Some(start2 + time::Duration::minutes(5))).await?;

        // End as KIA (State != survived)
        log_state_transition(&pool, raid2, "kia", Some(start2 + time::Duration::minutes(15))).await?;
        end_raid(&pool, raid2, Some(start2 + time::Duration::minutes(15)), None).await?;

        // Calculate Stats
        let stats = calculate_session_stats(&pool, session_id).await?;

        // Verification
        assert_eq!(stats.total_raids, 2, "Should have 2 raids");
        assert_eq!(stats.survived_raids, 1, "Should have 1 survival");
        assert_eq!(stats.survival_rate, 0.5, "50% survival rate");
        assert_eq!(stats.total_kills, 3, "Total kills should be 3");
        assert_eq!(stats.kd_ratio, 3.0, "3 kills / 1 death = 3.0 K/D");

        // Avg Duration: (30 + 15) / 2 = 22.5 minutes
        assert_eq!(stats.avg_raid_duration, time::Duration::minutes(22) + time::Duration::seconds(30));

        // Test Global Stats (should match since we only have 1 session)
        let global = calculate_global_stats(&pool, None).await?;
        assert_eq!(global.total_raids, 2);
        assert_eq!(global.kd_ratio, 3.0);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_game_mode_filtering() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();
        let session = create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;

        // Create PVE raid with 2 kills (survived)
        let pve_raid = create_raid(&pool, session, "Customs", CharacterType::PMC, GameMode::PVE, Some(base_time)).await?;
        add_kill(&pool, pve_raid, "scav", None, None, Some(base_time + time::Duration::minutes(5))).await?;
        add_kill(&pool, pve_raid, "scav", None, None, Some(base_time + time::Duration::minutes(10))).await?;
        log_state_transition(&pool, pve_raid, "survived", Some(base_time + time::Duration::minutes(20))).await?;
        end_raid(&pool, pve_raid, Some(base_time + time::Duration::minutes(20)), None).await?;

        // Create PVP raid with 1 kill (died)
        let pvp_raid = create_raid(&pool, session, "Factory", CharacterType::PMC, GameMode::PVP, Some(base_time +
    time::Duration::minutes(30))).await?;
        add_kill(&pool, pvp_raid, "pmc", None, None, Some(base_time + time::Duration::minutes(35))).await?;
        log_state_transition(&pool, pvp_raid, "died", Some(base_time + time::Duration::minutes(40))).await?;
        end_raid(&pool, pvp_raid, Some(base_time + time::Duration::minutes(40)), None).await?;

        // Test PVE filter
        let pve_stats = calculate_global_stats(&pool, Some(GameMode::PVE)).await?;
        assert_eq!(pve_stats.total_raids, 1);
        assert_eq!(pve_stats.total_kills, 2);
        assert_eq!(pve_stats.survival_rate, 1.0, "100% survival in PVE");

        // Test PVP filter
        let pvp_stats = calculate_global_stats(&pool, Some(GameMode::PVP)).await?;
        assert_eq!(pvp_stats.total_raids, 1);
        assert_eq!(pvp_stats.total_kills, 1);
        assert_eq!(pvp_stats.survival_rate, 0.0, "0% survival in PVP");

        pool.close().await;
        Ok(())
    }
    
    #[tokio::test]
    async fn test_perfect_survival_kd() -> Result<(), sqlx::Error> {
       let pool = setup_test_db().await?;
       let base_time = OffsetDateTime::now_utc();

       let session = create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;

       // Raid 1: Survived with 3 kills
       let raid1 = create_raid(&pool, session, "Customs", CharacterType::PMC, GameMode::PVE, Some(base_time)).await?;
       add_kill(&pool, raid1, "scav", None, None, Some(base_time + time::Duration::minutes(5))).await?;
       add_kill(&pool, raid1, "scav", None, None, Some(base_time + time::Duration::minutes(10))).await?;
       add_kill(&pool, raid1, "scav", None, None, Some(base_time + time::Duration::minutes(15))).await?;
       log_state_transition(&pool, raid1, "survived", Some(base_time + time::Duration::minutes(20))).await?;
       end_raid(&pool, raid1, Some(base_time + time::Duration::minutes(20)), None).await?;

       // Raid 2: Survived with 2 kills
       let raid2 = create_raid(&pool, session, "Woods", CharacterType::PMC, GameMode::PVE,
           Some(base_time + time::Duration::minutes(30))).await?;
       add_kill(&pool, raid2, "scav", None, None, Some(base_time + time::Duration::minutes(35))).await?;
       add_kill(&pool, raid2, "scav", None, None, Some(base_time + time::Duration::minutes(40))).await?;
       log_state_transition(&pool, raid2, "survived", Some(base_time + time::Duration::minutes(45))).await?;
       end_raid(&pool, raid2, Some(base_time + time::Duration::minutes(45)), None).await?;

       let stats = calculate_session_stats(&pool, session).await?;

       assert_eq!(stats.total_raids, 2);
       assert_eq!(stats.survived_raids, 2);
       assert_eq!(stats.survival_rate, 1.0, "100% survival");
       assert_eq!(stats.total_kills, 5);
       assert_eq!(stats.kd_ratio, 5.0, "Perfect survival: K/D should equal total kills");

       pool.close().await;
       Ok(())
    }

    #[tokio::test]
    async fn test_session_comparison() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();

        // Session 1: All-time baseline (1 raid, 1 kill, survived)
        let s1 = create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;
        let r1 = create_raid(&pool, s1, "Customs", CharacterType::PMC, GameMode::PVP, Some(base_time)).await?;
        add_kill(&pool, r1, "scav", None, None, Some(base_time)).await?;
        log_state_transition(&pool, r1, "survived", Some(base_time + Duration::minutes(10))).await?;
        end_raid(&pool, r1, Some(base_time + Duration::minutes(10)), None).await?;

        // Session 2: Current session (1 raid, 5 kills, died)
        let s2 = create_session(&pool, SessionType::Stream, None, Some(base_time + Duration::hours(1))).await?;
        let r2 = create_raid(&pool, s2, "Factory", CharacterType::PMC, GameMode::PVP, Some(base_time + Duration::hours(1))).await?;
        for _ in 0..5 {
            add_kill(&pool, r2, "pmc", None, None, Some(base_time + Duration::hours(1))).await?;
        }
        log_state_transition(&pool, r2, "died", Some(base_time + Duration::hours(1) + Duration::minutes(5))).await?;
        end_raid(&pool, r2, Some(base_time + Duration::hours(1) + Duration::minutes(5)), None).await?;

        let comparison = compare_session_to_mode_global(&pool, s2, None).await?;

        // Current (Session 2)
        assert_eq!(comparison.current.total_kills, 5);
        assert_eq!(comparison.current.survival_rate, 0.0);

        // All-time (Session 1 + Session 2)
        assert_eq!(comparison.all_time.total_raids, 2);
        assert_eq!(comparison.all_time.total_kills, 6);
        assert_eq!(comparison.all_time.survival_rate, 0.5);

        // Mode Stats for Session 2 (Only PVP raids)
        let modes = get_mode_stats_for_session(&pool, s2).await?;
        assert_eq!(modes.pvp.total_kills, 5);
        assert_eq!(modes.pve.total_raids, 0);

        pool.close().await;
        Ok(())
    }

    //Claude Test for edge cases
    #[tokio::test]
    async fn test_backwards_transition_queue_cancel() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();

        // Create session and raid
        let session_id = create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;
        let raid_id = create_raid(
            &pool,
            session_id,
            "Customs",
            CharacterType::PMC,
            GameMode::PVP,
            Some(base_time)
        ).await?;

        let mut time = base_time;

        // Normal flow: stash_management → pre_raid_setup → queuing
        log_state_transition(&pool, raid_id, "pre_raid_setup", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        log_state_transition(&pool, raid_id, "queuing", Some(time)).await?;
        time = time + time::Duration::minutes(3);

        // BACKWARDS: Cancel queue, return to stash_management
        log_state_transition(&pool, raid_id, "stash_management", Some(time)).await?;
        time = time + time::Duration::minutes(1);

        // Resume: Go back through the flow
        log_state_transition(&pool, raid_id, "pre_raid_setup", Some(time)).await?;
        time = time + time::Duration::minutes(1);

        log_state_transition(&pool, raid_id, "queuing", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        // Complete the raid
        log_state_transition(&pool, raid_id, "deploying_committed", Some(time)).await?;
        time = time + time::Duration::minutes(1);

        log_state_transition(&pool, raid_id, "raid_active", Some(time)).await?;
        time = time + time::Duration::minutes(20);

        log_state_transition(&pool, raid_id, "survived", Some(time)).await?;
        end_raid(&pool, raid_id, Some(time), None).await?;

        // Verify transitions were recorded
        let transitions = get_raid_transitions(&pool, raid_id).await?;
        assert_eq!(transitions.len(), 8, "Should have 8 transitions including backwards one");

        // Verify backwards transition exists
        let queue_to_stash = transitions.iter()
            .find(|t| t.from_state == Some("queuing".into()) && t.to_state == "stash_management");
        assert!(queue_to_stash.is_some(), "Should have backwards transition from queuing to stash");

        // Verify time in state handles multiple visits
        let state_times = calculate_time_in_state(&pool, raid_id).await?;

        // pre_raid_setup visited twice: 2 min + 1 min = 3 min total
        let pre_raid = state_times.iter()
            .find(|s| s.state == "pre_raid_setup")
            .expect("Should have pre_raid_setup");
        assert_eq!(pre_raid.duration, time::Duration::minutes(3),
            "Should accumulate time from both visits to pre_raid_setup");

        // queuing visited twice: 3 min + 2 min = 5 min total
        let queuing = state_times.iter()
            .find(|s| s.state == "queuing")
            .expect("Should have queuing");
        assert_eq!(queuing.duration, time::Duration::minutes(5),
            "Should accumulate time from both visits to queuing");

        // stash_management visited once (backwards transition): 1 min
        let stash = state_times.iter()
            .find(|s| s.state == "stash_management")
            .expect("Should have stash_management");
        assert_eq!(stash.duration, time::Duration::minutes(1),
            "Should track time in stash after queue cancel");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_reconnect_during_raid() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();

        let session_id = create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;
        let raid_id = create_raid(
            &pool,
            session_id,
            "Woods",
            CharacterType::PMC,
            GameMode::PVE,
            Some(base_time)
        ).await?;

        let mut time = base_time;

        // Normal flow to raid_active
        log_state_transition(&pool, raid_id, "pre_raid_setup", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        log_state_transition(&pool, raid_id, "queuing", Some(time)).await?;
        time = time + time::Duration::minutes(4);

        log_state_transition(&pool, raid_id, "deploying_committed", Some(time)).await?;
        time = time + time::Duration::minutes(2);

        log_state_transition(&pool, raid_id, "raid_active", Some(time)).await?;
        time = time + time::Duration::minutes(10);

        // Get 2 kills before disconnect
        add_kill(&pool, raid_id, "scav", Some("AK-74".into()), Some(false), Some(time)).await?;
        time = time + time::Duration::minutes(3);
        add_kill(&pool, raid_id, "pmc", Some("M4A1".into()), Some(true), Some(time)).await?;
        time = time + time::Duration::minutes(2);

        // DISCONNECT - internet drops
        log_state_transition(&pool, raid_id, "disconnected", Some(time)).await?;
        time = time + time::Duration::minutes(5); // 5 min to reconnect

        // RECONNECT - back to raid_active
        log_state_transition(&pool, raid_id, "raid_active", Some(time)).await?;
        time = time + time::Duration::minutes(8);

        // Get 1 more kill after reconnect
        add_kill(&pool, raid_id, "scav", Some("SKS".into()), Some(false), Some(time)).await?;
        time = time + time::Duration::minutes(2);

        // Extract successfully
        log_state_transition(&pool, raid_id, "raid_ending", Some(time)).await?;
        time = time + time::Duration::minutes(1);

        log_state_transition(&pool, raid_id, "survived", Some(time)).await?;
        end_raid(&pool, raid_id, Some(time), Some("Bridge".into())).await?;

        // Verify transitions
        let transitions = get_raid_transitions(&pool, raid_id).await?;
        assert_eq!(transitions.len(), 8, "Should have 8 transitions including disconnect/reconnect");

        // Verify disconnect exists
        let disconnect = transitions.iter()
            .find(|t| t.to_state == "disconnected");
        assert!(disconnect.is_some(), "Should have disconnected state");

        // Verify reconnect (raid_active appears twice)
        let raid_active_count = transitions.iter()
            .filter(|t| t.to_state == "raid_active")
            .count();
        assert_eq!(raid_active_count, 2, "Should have 2 transitions to raid_active (initial + reconnect)");

        // Verify time in state
        let state_times = calculate_time_in_state(&pool, raid_id).await?;

        // raid_active visited twice: 15 min (before disconnect) + 10 min (after reconnect) = 25 min
        let raid_active = state_times.iter()
            .find(|s| s.state == "raid_active")
            .expect("Should have raid_active");
        assert_eq!(raid_active.duration, time::Duration::minutes(25),
            "Should accumulate time in raid_active from both sessions");

        // disconnected: 5 min
        let disconnected = state_times.iter()
            .find(|s| s.state == "disconnected")
            .expect("Should have disconnected state");
        assert_eq!(disconnected.duration, time::Duration::minutes(5),
            "Should track disconnection time");

        // Verify all kills were recorded (3 total)
        let kills = get_kills_for_raid(&pool, raid_id).await?;
        assert_eq!(kills.len(), 3, "Should have 3 kills across disconnect");

        pool.close().await;
        Ok(())
    }
}
