pub(crate) mod action;
pub(crate) mod command;

fn main() {
    use clap::Parser;

    let args = command::Command::parse();

    match args.action {
        command::SubCommand::Run(command) => {
            action::execute(command.flags);
        }
    }
}
