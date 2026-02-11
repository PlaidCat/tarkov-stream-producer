use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")] // Ensure serde matches sqlx's lowercase
pub enum GameMode {
    PVE,
    PVP,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")] // Ensure serde matches sqlx's lowercase
pub enum SessionType {
    Stream,
    Practice,
    Casual,
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
}
