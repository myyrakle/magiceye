use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CheckType {
    CommentOfColumn,
    CommentOfTable,
    TypeOfColumn,
    IndexOfTable,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Language {
    English,
    Korean,
}

impl Language {
    pub fn list() -> Vec<Self> {
        vec![Self::English, Self::Korean]
    }

    pub fn next(&self) -> Self {
        match self {
            Self::English => Self::Korean,
            Self::Korean => Self::English,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::English => Self::Korean,
            Self::Korean => Self::English,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DatabaseType {
    Postgres,
    Mysql,
}

impl DatabaseType {
    pub fn list() -> Vec<Self> {
        vec![Self::Postgres, Self::Mysql]
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Postgres => Self::Mysql,
            Self::Mysql => Self::Postgres,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Postgres => Self::Mysql,
            Self::Mysql => Self::Postgres,
        }
    }
}

impl Default for DatabaseType {
    fn default() -> Self {
        Self::Postgres
    }
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub database_pairs: Vec<DatabasePair>,
    pub default_database_pair: Option<DatabasePair>,
    pub current_language: Language,
    pub ignore_list: Vec<CheckType>,
}
