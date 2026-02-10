use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::db::tests::setup_test_db;

    #[tokio::test]
    async fn test_app_state_new() {
        let pool = setup_test_db().await.expect("setup db");
        let state = AppState::new(pool.clone());
        assert!(!state.pool.is_closed());
    }
}
