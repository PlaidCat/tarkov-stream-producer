use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================
// Enums
// ============================================================
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")] // Ensure serde matches sqlx's lowercase
pub enum CharacterType {
    PMC,
    Scav,
}

impl fmt::Display for CharacterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharacterType::PMC => write!(f, "PMC"),
            CharacterType::Scav => write!(f, "Scav"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")] // Ensure serde matches sqlx's lowercase
pub enum GameMode {
    PVE,
    PVP,
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameMode::PVE => write!(f, "PVE"),
            GameMode::PVP => write!(f, "PVP"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")] // Ensure serde matches sqlx's lowercase
pub enum SessionType {
    Stream,
    Practice,
    Casual,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionType::Stream => write!(f, "Stream"),
            SessionType::Practice => write!(f, "Practice"),
            SessionType::Casual => write!(f, "Casual"),
        }
    }
}

// ============================================================
// Structs
// ============================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StreamSession {
    pub session_id: i64,
    pub started_at: OffsetDateTime,
    pub ended_at: Option<OffsetDateTime>,
    pub session_type: Option<SessionType>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Raid {
    pub raid_id: i64,
    pub session_id: i64,
    pub started_at: OffsetDateTime,
    pub ended_at: Option<OffsetDateTime>,
    pub map_name: String,
    pub character_type: CharacterType,
    pub game_mode: GameMode,
    pub current_state: String, //String for extensibility
    pub extract_location: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RaidStateTransition {
    pub transition_id: i64,
    pub raid_id: i64,
    pub from_state: Option<String>,
    pub to_state: String,
    pub transitioned_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Kill {
    pub kill_id: i64,
    pub raid_id: i64,
    pub killed_at: OffsetDateTime,
    pub enemy_type: String, //String for extensibility
    pub weapon_used: Option<String>,
    pub headshot: Option<bool>,
    pub distance_meters: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_type_display() {
        assert_eq!(CharacterType::PMC.to_string(), "PMC");
        assert_eq!(CharacterType::Scav.to_string(), "Scav");
    }

    #[test]
    fn test_game_mode_display() {
        assert_eq!(GameMode::PVE.to_string(), "PVE");
        assert_eq!(GameMode::PVP.to_string(), "PVP");
    }

    #[test]
    fn test_session_type_display() {
        assert_eq!(SessionType::Stream.to_string(), "Stream");
        assert_eq!(SessionType::Practice.to_string(), "Practice");
        assert_eq!(SessionType::Casual.to_string(), "Casual");
    }
}
