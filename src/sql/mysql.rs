use sqlx::mysql::MySqlPoolOptions;

use super::ConnectionPool;

pub async fn get_connection_pool(connection_url: &str) -> Result<ConnectionPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(ConnectionPool::MySQL(pool))
}
