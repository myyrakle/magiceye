use crate::{
    config::{DatabasePair, DatabaseType},
    sql::{mysql, postgres, ConnectionPool},
};

pub async fn connect_database(
    database_pair: &DatabasePair,
) -> anyhow::Result<(ConnectionPool, ConnectionPool)> {
    let base_connection_url = &database_pair.base_connection;
    let target_connection_url = &database_pair.target_connection;
    let database_type = &database_pair.database_type;

    let base_connection_pool = match database_type {
        DatabaseType::Postgres => postgres::get_connection_pool(base_connection_url).await,
        DatabaseType::Mysql => mysql::get_connection_pool(base_connection_url).await,
    };

    let target_connection_pool = match database_type {
        DatabaseType::Postgres => postgres::get_connection_pool(target_connection_url).await,
        DatabaseType::Mysql => mysql::get_connection_pool(target_connection_url).await,
    };

    let base_connection_pool = match base_connection_pool {
        Ok(pool) => pool,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to connect to base database: {:?}",
                error
            ));
        }
    };

    let target_connection_pool = match target_connection_pool {
        Ok(pool) => {
            //println!(">> connected to target database");
            pool
        }
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to connect to target database: {:?}",
                error
            ));
        }
    };

    Ok((base_connection_pool, target_connection_pool))
}
