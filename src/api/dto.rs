use serde::{Deserialize, Serialize};
use crate::models::{CharacterType, GameMode, SessionType};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub session_type: SessionType,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRaidRequest {
    pub map_name: String,
    pub character_type: CharacterType,
    pub game_mode: GameMode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StateTransitionRequest {
    pub to_state: String,
    pub transitioned_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EndRaidRequest {
    pub final_state: String,
    pub extract_location: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RaidResponse {
    pub raid_id: i64,
    pub session_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub map_name: String,
    pub character_type: CharacterType,
    pub game_mode: GameMode,
    pub current_state: String,
    pub extract_location: Option<String>,
}

impl From<crate::models::Raid> for RaidResponse {
    fn from(r: crate::models::Raid) -> Self {
        RaidResponse {
            raid_id: r.raid_id,
            session_id: r.session_id,
            started_at: r.started_at.to_string(),
            ended_at: r.ended_at.map(|t| t.to_string()),
            map_name: r.map_name,
            character_type: r.character_type,
            game_mode: r.game_mode,
            current_state: r.current_state,
            extract_location: r.extract_location,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddKillRequest {
    pub enemy_type: String,
    pub weapon_used: Option<String>,
    pub headshot: Option<bool>,
    pub distance_meters: Option<f64>,
    pub killed_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchKillsRequest {
    pub kills: Vec<AddKillRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KillResponse {
    pub kill_id: i64,
    pub raid_id: i64,
    pub killed_at: String,
    pub enemy_type: String,
    pub weapon_used: Option<String>,
    pub headshot: Option<bool>,
    pub distance_meters: Option<f64>,
}

impl From<crate::models::Kill> for KillResponse {
    fn from(k: crate::models::Kill) -> Self {
        KillResponse {
            kill_id: k.kill_id,
            raid_id: k.raid_id,
            killed_at: k.killed_at.to_string(),
            enemy_type: k.enemy_type,
            weapon_used: k.weapon_used,
            headshot: k.headshot,
            distance_meters: k.distance_meters,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VibeKillRequest {
    pub kills_text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionStatsResponse {
    pub total_raids: i64,
    pub survived_raids: i64,
    pub survival_rate: f64,
    pub total_kills: i64,
    pub kd_ratio: f64,
    pub avg_raid_duration_seconds: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StateTimeResponse {
    pub state: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RaidStatsResponse {
    pub raid_id: i64,
    pub state_durations: Vec<StateTimeResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_request_deserialization() {
        let json = r#"{"session_type": "stream", "notes": "Test session"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.session_type, SessionType::Stream);
        assert_eq!(req.notes, Some("Test session".into()));
    }

    #[test]
    fn test_create_session_request_notes_optional() {
        let json = r#"{"session_type": "practice"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.session_type, SessionType::Practice);
        assert_eq!(req.notes, None);
    }

    #[test]
    fn test_create_raid_request_deserialization() {
        let json = r#"{"map_name": "Customs", "character_type": "pmc", "game_mode": "pve"}"#;
        let req: CreateRaidRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.map_name, "Customs");
        assert_eq!(req.character_type, CharacterType::PMC);
        assert_eq!(req.game_mode, GameMode::PVE);
    }

    #[test]
    fn test_state_transition_request_deserialization() {
        let json = r#"{"to_state": "queue", "transitioned_at": "2026-02-13T12:00:00Z"}"#;
        let req: StateTransitionRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.to_state, "queue");
        assert_eq!(req.transitioned_at, Some("2026-02-13T12:00:00Z".into()));
    }

    #[test]
    fn test_end_raid_request_deserialization() {
        let json = r#"{"final_state": "survived", "extract_location": "Crossroads"}"#;
        let req: EndRaidRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.final_state, "survived");
        assert_eq!(req.extract_location, Some("Crossroads".into()));
        assert_eq!(req.ended_at, None);
    }

    #[test]
    fn test_raid_response_serialization() {
        let resp = RaidResponse {
            raid_id: 101,
            session_id: 1,
            started_at: "2026-02-13T10:00:00Z".to_string(),
            ended_at: None,
            map_name: "Interchange".to_string(),
            character_type: CharacterType::Scav,
            game_mode: GameMode::PVP,
            current_state: "in_raid".to_string(),
            extract_location: None,
        };

        let json = serde_json::to_string(&resp).unwrap();

        // check key fields
        assert!(json.contains(r#""raid_id":101"#));
        assert!(json.contains(r#""map_name":"Interchange""#));
        assert!(json.contains(r#"character_type":"scav""#));
    }

    #[test]
    fn test_add_kill_request_deserialization() {
        let json = r#"{"enemy_type": "scav", "weapon_used": "M4A1", "headshot": true, "killed_at": "2026-02-22T10:00:00Z"}"#;
        let req: AddKillRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.enemy_type, "scav");
        assert_eq!(req.weapon_used, Some("M4A1".to_string()));
        assert_eq!(req.headshot, Some(true));
        assert_eq!(req.killed_at, Some("2026-02-22T10:00:00Z".to_string()));
    }

    #[test]
    fn test_add_kill_request_optional_fields() {
        let json = r#"{"enemy_type": "pmc"}"#;
        let req: AddKillRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.enemy_type, "pmc");
        assert_eq!(req.weapon_used, None);
        assert_eq!(req.headshot, None);
    }

    #[test]
    fn test_batch_kills_request_deserialization() {
        let json = r#"{
            "kills": [
                {"enemy_type": "scav", "headshot": true},
                {"enemy_type": "pmc", "weapon_used": "AK74"}
            ]
        }"#;
        let req: BatchKillsRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.kills.len(), 2);
        assert_eq!(req.kills[0].enemy_type, "scav");
        assert_eq!(req.kills[1].weapon_used, Some("AK74".to_string()));
    }

    #[test]
    fn test_kill_response_serialization() {
        let resp = KillResponse {
            kill_id: 1,
            raid_id: 42,
            killed_at: "2026-02-22T10:00:00Z".to_string(),
            enemy_type: "scav".to_string(),
            weapon_used: Some("M4A1".to_string()),
            headshot: Some(true),
            distance_meters: Some(42.5),
        };

        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""kill_id":1"#));
        assert!(json.contains(r#""enemy_type":"scav""#));
        assert!(json.contains(r#""headshot":true"#));
    }

    #[test]
    fn test_session_stats_response_serialization() {
        let resp = SessionStatsResponse {
            total_raids: 5,
            survived_raids: 3,
            survival_rate: 0.6,
            total_kills: 15,
            kd_ratio: 7.5,
            avg_raid_duration_seconds: 1200.5,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""total_raids":5"#));
        assert!(json.contains(r#""kd_ratio":7.5"#));
        assert!(json.contains(r#""avg_raid_duration_seconds":1200.5"#));
    }

    #[test]
    fn test_raid_stats_response_serialization() {
        let resp = RaidStatsResponse {
            raid_id: 1,
            state_durations: vec![
                StateTimeResponse {
                    state: "queuing".to_string(),
                    duration_seconds: 300.0,
                },
                StateTimeResponse {
                    state: "raid_active".to_string(),
                    duration_seconds: 1800.0,
                },
            ],
        };

        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""raid_id":1"#));
        assert!(json.contains(r#""state":"queuing""#));
        assert!(json.contains(r#""duration_seconds":300.0"#));
    }
}
