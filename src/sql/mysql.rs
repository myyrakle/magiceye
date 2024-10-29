use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use super::ConnectionPool;

pub async fn get_connection_pool(connection_url: &str) -> Result<ConnectionPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(ConnectionPool::MySQL(pool))
}

pub async fn get_table_list(pool: &Pool<MySql>) -> Vec<String> {
    let table_list = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = DATABASE()
    "#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to fetch table list");

    table_list
        .into_iter()
        .map(|(table_name,)| table_name)
        .collect()
}
