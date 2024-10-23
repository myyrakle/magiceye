mod command;

fn main() {
    use clap::Parser;

    let args = command::Command::parse();

    match args.action {
        command::SubCommand::Run(run) => {
            println!("{:?}", run);
        }
    }
}
