use std::sync::{Arc, Mutex};

use duckdb::{AccessMode, Config, Connection};

use crate::routes::ApiError;

#[derive(Clone)]
pub struct Db(Arc<Mutex<Connection>>);

impl Db {
    pub fn open(path: &str) -> duckdb::Result<Self> {
        let config = Config::default().access_mode(AccessMode::ReadOnly)?;
        let conn = Connection::open_with_flags(path, config)?;
        Ok(Self(Arc::new(Mutex::new(conn))))
    }

    /// Runs `f` on its own connection on a blocking thread, so slow queries
    /// don't hold up the async runtime.
    pub async fn run<T, F>(&self, f: F) -> Result<T, ApiError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> duckdb::Result<T> + Send + 'static,
    {
        let conn = self.0.lock().unwrap().try_clone()?;
        tokio::task::spawn_blocking(move || f(&conn))
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .map_err(ApiError::from)
    }
}
