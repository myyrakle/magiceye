use serde::Deserialize;

use clap::Args;

#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct CommandFlags {}

#[derive(Clone, Debug, Args)]
#[clap(name = "run", about = "run magiceye")]
pub struct Command {
    #[clap(flatten)]
    pub flags: CommandFlags,
}
