use serde::Deserialize;

use clap::Args;

#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct CommandFlags {}

#[derive(Clone, Debug, Args)]
#[clap(name = "init", about = "initialize magiceye config")]
pub struct Command {
    #[clap(flatten)]
    pub flags: CommandFlags,
}
