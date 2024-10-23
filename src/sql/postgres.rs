use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn get_connection_pool(connection_url: &str) -> Pool<Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await
        .expect("Failed to create connection pool");

    pool
}
