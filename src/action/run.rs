use crate::{command::run::CommandFlags, platform_specific::get_config};

pub fn execute(flags: CommandFlags) {
    log::info!("execute action: run");

    let config = get_config();

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);
}
