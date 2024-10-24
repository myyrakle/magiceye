use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CheckType {
    CommentOfColumn,
    CommentOfTable,
    TypeOfColumn,
    IndexOfTable,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Language {
    English,
    Korean,
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DatabaseType {
    Postgres,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabasePair {
    pub name: String,
    pub database_type: DatabaseType,
    pub base_connection: String,
    pub target_connection: String,
}

impl Default for DatabasePair {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            database_type: DatabaseType::Postgres,
            base_connection: String::new(),
            target_connection: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub database_pairs: Vec<DatabasePair>,
    pub default_database_pair: Option<DatabasePair>,
    pub current_language: Language,
    pub ignore_list: Vec<CheckType>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_pairs: vec![],
            current_language: Language::default(),
            default_database_pair: None,
            ignore_list: vec![],
        }
    }
}
