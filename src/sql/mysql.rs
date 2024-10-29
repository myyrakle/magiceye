use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub async fn get_connection_pool(connection_url: &str) -> Result<Pool<MySql>, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(pool)
}
