use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn get_connection_pool(connection_url: &str) -> Pool<Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await
        .expect("Failed to create connection pool");

    pool
}

pub async fn get_table_list(pool: &Pool<Postgres>) -> Vec<String> {
    let table_list = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
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
