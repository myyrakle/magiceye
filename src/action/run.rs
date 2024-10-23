use crate::{command::run::CommandFlags, platform_specific::get_config};

pub fn execute(option: CommandFlags) {
    let config = get_config();

    println!("config: {:?}", config);

    println!("run: {:?}", option);
}
