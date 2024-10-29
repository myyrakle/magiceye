pub mod mysql;
pub mod postgres;

#[derive(Debug)]
pub enum ConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
    MySQL(sqlx::Pool<sqlx::MySql>),
}

#[derive(Debug, Default)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub default: String,
    pub nullable: bool,
    pub comment: String,
    pub is_auto_increment: bool, // MYSQL Only
}

#[derive(Debug)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub predicate: String,
    pub is_unique: bool,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub comment: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
}
