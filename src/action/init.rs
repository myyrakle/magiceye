use crate::{command::init::CommandFlags, platform_specific::get_config};

pub async fn execute(flags: CommandFlags) {
    log::info!("execute action: init");

    let config = get_config();

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);
}
