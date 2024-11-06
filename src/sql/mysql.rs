use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::sql::{Column, ForeignKey, Index};

use super::{ConnectionPool, Table};

pub async fn get_connection_pool(connection_url: &str) -> anyhow::Result<ConnectionPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(ConnectionPool::MySQL(pool))
}

pub async fn get_table_list(pool: &Pool<MySql>) -> anyhow::Result<Vec<String>> {
    let table_list = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = DATABASE()
    "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(table_list
        .into_iter()
        .map(|(table_name,)| table_name)
        .collect())
}

pub async fn describe_table(pool: &Pool<MySql>, table_name: &str) -> anyhow::Result<Table> {
    log::debug!("describe table: {table_name}");

    // 1. 컬럼 리스트 정보 조회
    let query_result = sqlx::query_as::<_, (String, String, String, i32, String, String)>(
        r#"
        SELECT 
            column_name, 
            column_type, 
            coalesce(column_default, ''), is_nullable = 'YES',
            column_comment, 
            coalesce(extra, '')
        FROM 
            information_schema.columns
        WHERE 
            table_name = ?
            AND table_schema = DATABASE()
    "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let columns = query_result
        .into_iter()
        .map(
            |(name, data_type, default, nullable, comment, extra)| Column {
                name,
                data_type,
                default,
                nullable: nullable == 1,
                comment,
                is_auto_increment: extra.contains("auto_increment"),
            },
        )
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
    .await?;

    let indexes = query_result
        .into_iter()
        .map(|(name, columns, is_unique)| Index {
            name,
            columns: columns.split(',').map(|s| s.to_string()).collect(),
            is_unique,
            predicate: "".to_string(),
        })
        .collect();

    // 3. 테이블에 속한 외래키 목록 조회
    let query_result = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
            SELECT 
                kcu.constraint_name,
                kcu.column_name,
                kcu.referenced_table_name,
                kcu.referenced_column_name
            FROM 
                information_schema.key_column_usage kcu
            JOIN 
                information_schema.referential_constraints rc
            ON 
                kcu.constraint_name = rc.constraint_name
            WHERE 1=1
                AND kcu.table_name = ?
                AND kcu.table_schema = DATABASE()
                AND rc.constraint_schema = DATABASE()
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let mut constraints = vec![];

    for (name, column, foreign_table_name, foreign_column_name) in query_result {
        constraints.push(
            ForeignKey {
                name,
                column: vec![column],
                foreign_column: super::SelectColumn {
                    table_name: foreign_table_name,
                    column_name: foreign_column_name,
                },
            }
            .into(),
        );
    }

    let table = Table {
        name: table_name.to_string(),
        comment: "".to_string(), // TODO: 테이블 comment 조회
        columns,
        indexes,
        constraints,
    };

    Ok(table)
}
