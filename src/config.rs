use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseType {
    Postgres,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabasePair {
    pub name: String,
    pub database_type: DatabaseType,
    pub base_connection: String,
    pub target_connection: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_pairs: Vec<DatabasePair>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_pairs: vec![],
        }
    }
}
