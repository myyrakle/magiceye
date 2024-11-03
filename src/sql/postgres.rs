use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::sql::{Column, ForeignKey, Index};

use super::{ConnectionPool, Table};

pub async fn ping(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    pool.acquire().await?;

    Ok(())
}

pub async fn get_connection_pool(connection_url: &str) -> Result<ConnectionPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(ConnectionPool::Postgres(pool))
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

fn format_type(data_type: &str, character_maximum_length: i32) -> String {
    match data_type {
        "character varying" => format!("varchar({character_maximum_length})"),
        _ => data_type.to_string(),
    }
}

pub async fn describe_table(pool: &Pool<Postgres>, table_name: &str) -> Table {
    log::debug!("describe table: {table_name}");

    // 1. 컬럼 리스트 정보 조회
    let query_result = sqlx::query_as::<_, (String, String, i32, String, String, String)>(
        r#"
        SELECT 
            c.column_name, 
            c.data_type, coalesce(c.character_maximum_length, 0) as character_maximum_length,
            coalesce(c.column_default, ''), c.is_nullable, 
            coalesce(pgd.description, '') as comment
        FROM 
            information_schema.columns c
        LEFT JOIN 
            pg_catalog.pg_description pgd 
        ON pgd.objsubid = c.ordinal_position

        AND 
            pgd.objoid = (
                SELECT oid 
                FROM pg_catalog.pg_class 
                WHERE relname = c.table_name
            )
        WHERE c.table_name = $1
    "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch column list");

    let columns = query_result
        .into_iter()
        .map(
            |(name, data_type, character_maximum_length, default, nullable, comment)| Column {
                name,
                data_type: format_type(&data_type, character_maximum_length),
                default,
                nullable: nullable == "YES",
                comment,
                ..Default::default()
            },
        )
        .collect();

    // 2. 테이블 메타 정보 조회
    let query_result = sqlx::query_as::<_, (String,)>(
        r#"
            SELECT pgd.description
            FROM pg_catalog.pg_description pgd
            JOIN pg_catalog.pg_class c ON c.oid = pgd.objoid
            WHERE c.relname = $1
            AND pgd.objsubid = 0
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch column list");

    let table_comment = query_result
        .first()
        .map(|(comment,)| comment.to_owned())
        .unwrap_or_default();

    // 3. 테이블에 속한 인덱스 목록 조회
    let query_result = sqlx::query_as::<_, (String, String, bool, String)>(
        r#"
            SELECT
                i.relname AS index_name,
                string_agg(a.attname, ',' ORDER BY array_position(ix.indkey, a.attnum)) AS columns,
                ix.indisunique AS is_unique,
                coalesce(pg_get_expr(ix.indpred, ix.indrelid), '') AS predicate
            FROM
                pg_class t,
                pg_class i,
                pg_index ix,
                pg_attribute a
            WHERE
                t.oid = ix.indrelid
                AND i.oid = ix.indexrelid
                AND a.attrelid = t.oid
                AND a.attnum = ANY(ix.indkey)
                AND i.relname IN (
                    SELECT indexname
                    FROM pg_indexes
                    WHERE tablename = $1
                )
            GROUP BY
                i.relname, ix.indisunique, ix.indpred, ix.indrelid;
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch index list");

    let indexes = query_result
        .into_iter()
        .map(|(name, columns, is_unique, predicate)| Index {
            name,
            columns: columns.split(',').map(|s| s.to_string()).collect(),
            is_unique,
            predicate,
        })
        .collect();

    let mut constraints = vec![];

    // 4. 테이블에 속한 외래키 목록 조회
    let query_result = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
            SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM
                information_schema.table_constraints AS tc
                JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
            WHERE
                tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_name = $1;
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch foreign key list");

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

    Table {
        name: table_name.to_string(),
        comment: table_comment,
        columns,
        indexes,
        constraints,
    }
}
