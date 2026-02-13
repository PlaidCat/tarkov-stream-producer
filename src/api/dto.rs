use serde::{Deserialize, Serialize};
use crate::models::SessionType;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub session_type: SessionType,
    pub notes: Option<String>,
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
}
