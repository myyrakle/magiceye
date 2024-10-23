use crate::{
    command::run::CommandFlags, platform_specific::get_config, sql::postgres::get_connection_pool,
};

pub async fn execute(flags: CommandFlags) {
    log::info!("execute action: run");

    let config = get_config();

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);
}
