use serde::Deserialize;

use clap::Args;

#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOptions {}

#[derive(Clone, Debug, Args)]
#[clap(name = "run", about = "run magiceye")]
pub struct Command {
    #[clap(flatten)]
    pub value: ConfigOptions,
}
