pub(crate) mod action;
pub(crate) mod command;
pub mod config;
pub mod platform_specific;
pub mod sql;

#[tokio::main]
async fn main() {
    env_logger::init();

    use clap::Parser;

    let args = command::Command::parse();

    match args.action {
        command::SubCommand::Run(command) => {
            action::run::execute(command.flags).await;
        }
        command::SubCommand::Init(command) => {
            unimplemented!();
        }
    }
}
