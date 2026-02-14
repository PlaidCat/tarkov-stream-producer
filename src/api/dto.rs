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
}
