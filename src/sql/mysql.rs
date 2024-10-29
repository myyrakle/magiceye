use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::sql::{Column, Index};

use super::{ConnectionPool, Table};

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

pub async fn describe_table(pool: &Pool<MySql>, table_name: &str) -> Table {
    log::debug!("describe table: {table_name}");

    // 1. 컬럼 리스트 정보 조회
    let query_result = sqlx::query_as::<_, (String, String, String, i32, String)>(
        r#"
        SELECT 
            column_name, 
            column_type, 
            coalesce(column_default, ''), is_nullable = 'YES',
            column_comment
        FROM 
            information_schema.columns
        WHERE 
            table_name = ?
            AND table_schema = DATABASE()
    "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch column list");

    let columns = query_result
        .into_iter()
        .map(|(name, data_type, default, nullable, comment)| Column {
            name,
            data_type: data_type,
            default,
            nullable: nullable == 1,
            comment,
        })
        .collect();

    // 2. 테이블에 속한 인덱스 목록 조회
    let query_result = sqlx::query_as::<_, (String, String, bool)>(
        r#"
            SELECT 
                index_name, 
                GROUP_CONCAT(column_name ORDER BY seq_in_index) AS columns,
                !non_unique
            FROM 
                information_schema.statistics
            WHERE 
                table_name = ?
                AND table_schema = DATABASE()
            GROUP BY 
                index_name, non_unique
            ORDER BY 
                index_name
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch index list");

    let indexes = query_result
        .into_iter()
        .map(|(name, columns, is_unique)| Index {
            name,
            columns: columns.split(',').map(|s| s.to_string()).collect(),
            is_unique,
            predicate: "".to_string(),
        })
        .collect();

    Table {
        name: table_name.to_string(),
        comment: "".to_string(), // TODO: 테이블 comment 조회
        columns,
        indexes,
    }
}
