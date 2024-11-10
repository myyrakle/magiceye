use std::collections::HashMap;

use crate::{
    action::run::{
        tui::{FetchingTableList, ProgressEvent},
        SenderContext,
    },
    sql::{mysql, postgres, ConnectionPool, Table},
};

pub async fn get_table_list(
    context: &SenderContext,
    connection_pool: &ConnectionPool,
) -> anyhow::Result<HashMap<String, Table>> {
    let table_list_result = match connection_pool {
        ConnectionPool::Postgres(ref pool) => postgres::get_table_list(pool).await,
        ConnectionPool::MySQL(ref pool) => mysql::get_table_list(pool).await,
    };

    let table_list = match table_list_result {
        Ok(list) => list,
        Err(error) => {
            return Err(anyhow::anyhow!("failed to get table list: {:?}", error));
        }
    };

    let mut table_map = HashMap::new();

    for (i, table_name) in table_list.iter().enumerate() {
        _ = context
            .event_sender
            .send(ProgressEvent::FetchingTableList(FetchingTableList {
                total_count: Some(table_list.len()),
                current_count: i + 1,
            }));

        let table_result = match connection_pool {
            ConnectionPool::Postgres(ref pool) => postgres::describe_table(pool, table_name).await,
            ConnectionPool::MySQL(ref pool) => mysql::describe_table(pool, table_name).await,
        };

        let table = match table_result {
            Ok(table) => table,
            Err(error) => {
                return Err(anyhow::anyhow!("failed to describe table: {:?}", error));
            }
        };

        table_map.insert(table_name.to_owned(), table);
    }

    _ = context
        .event_sender
        .send(ProgressEvent::FetchingTableList(FetchingTableList {
            total_count: Some(table_list.len()),
            current_count: table_list.len(),
        }));

    Ok(table_map)
}
