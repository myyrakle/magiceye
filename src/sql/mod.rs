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

#[derive(Debug, PartialEq)]
pub struct ForeignKey {
    pub name: String,
    pub column: Vec<String>,
    pub foreign_column: SelectColumn,
}

impl From<ForeignKey> for Constraint {
    fn from(fk: ForeignKey) -> Self {
        Constraint::ForeignKey(fk)
    }
}

#[derive(Debug)]
pub enum Constraint {
    ForeignKey(ForeignKey),
}

#[derive(Debug, PartialEq)]
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

// foreign key 관련 메서드
impl Table {
    #[allow(clippy::unnecessary_filter_map)]
    pub fn foreign_keys(&self) -> Vec<&ForeignKey> {
        self.constraints
            .iter()
            .filter_map(|c| match c {
                Constraint::ForeignKey(fk) => Some(fk),
                // _ => None,
            })
            .collect()
    }

    pub fn find_foreign_key_by_key_name(&self, key_name: &str) -> Option<&ForeignKey> {
        self.foreign_keys()
            .iter()
            .find(|fk| fk.name == key_name)
            .cloned()
    }
}
