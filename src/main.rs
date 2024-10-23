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
            action::execute(command.flags).await;
        }
    }
}
