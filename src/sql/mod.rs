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
pub struct ForeignKey {
    pub name: String,
    pub column: Vec<String>,
    pub foreign_column: Option<SelectColumn>,
}

#[derive(Debug)]
pub enum Constraint {
    ForeignKey(ForeignKey),
}

#[derive(Debug)]
pub struct SelectColumn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Debug, Default)]
pub struct Table {
    pub name: String,
    pub comment: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub constraints: Vec<Constraint>,
}
