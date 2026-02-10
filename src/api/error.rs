pub enum AppError { 
    DatabaseError(sqlx::Error),
    NotFound(String),
    Conflict(String),
    ValidationError(String),
    BadRequest(String),
}

impl AppError {
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            AppError::DatabaseError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => http::StatusCode::NOT_FOUND,
            AppError::Conflict(_) => http::StatusCode::CONFLICT,
            AppError::ValidationError(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn json_body(&self) -> serde_json::Value {
        use serde_json::json;

        let (error_type, message) = match self {
            AppError::DatabaseError(e) => ("database_error", e.to_string()),
            AppError::NotFound(msg) => ("not_found", msg.clone()),
            AppError::Conflict(msg) => ("conflict", msg.clone()),
            AppError::ValidationError(msg) => ("validation_error", msg.clone()),
            AppError::BadRequest(msg) => ("bad_request", msg.clone()),
        };

        json!({
            "error": message,
            "type": error_type
        })
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = self.json_body();

        (status, axum::Json(body)).into_response()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_variants_exist() {
        let _ = AppError::NotFound("test".to_string());
        let _ = AppError::Conflict("test".to_string());
        let _ = AppError::ValidationError("test".to_string());
        let _ = AppError::BadRequest("test".to_string());
    }

    #[test]
    fn test_error_status_code() {
        use http::StatusCode;

        assert_eq!(AppError::NotFound("x".into()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::Conflict("x".into()).status_code(), StatusCode::CONFLICT);
        assert_eq!(AppError::ValidationError("x".into()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(AppError::BadRequest("x".into()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::DatabaseError(sqlx::Error::RowNotFound).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_json_body() {

        let error = AppError::NotFound("Session not found".to_string());
        let body = error.json_body();

        assert_eq!(body["error"], "Session not found");
        assert_eq!(body["type"], "not_found");

        let validation_error = AppError::ValidationError("Invalid raid ID".to_string());
        let body = validation_error.json_body();

        assert_eq!(body["error"], "Invalid raid ID");
        assert_eq!(body["type"], "validation_error");
    }

    #[test]
    fn test_into_response() {
        use axum::response::IntoResponse;
        use http::StatusCode;

        let error = AppError::NotFound("Session not found".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
