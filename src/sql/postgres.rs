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

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub default: String,
    pub nullable: bool,
    pub comment: String,
}

#[derive(Debug)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub predicate: String,
    pub is_unique: bool,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub comment: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
}

pub async fn describe_table(pool: &Pool<Postgres>, table_name: &str) -> Table {
    log::debug!("describe table: {table_name}");

    // 1. 컬럼 리스트 정보 조회
    let query_result = sqlx::query_as::<_, (String, String, String, String, String)>(
        r#"
        SELECT 
            c.column_name, c.data_type, coalesce(c.column_default, ''), c.is_nullable, 
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
        .map(|(name, data_type, default, nullable, comment)| Column {
            name,
            data_type,
            default,
            nullable: nullable == "YES",
            comment,
        })
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
        .get(0)
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
            is_unique: is_unique,
            predicate: predicate,
        })
        .collect();

    let table = Table {
        name: table_name.to_string(),
        comment: table_comment,
        columns,
        indexes,
    };

    table
}
