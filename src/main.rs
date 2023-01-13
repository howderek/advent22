use clap::Parser;

mod challenges;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Challenge Day
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Blizzard Basin
    Day24(challenges::day24::Args),
    Day23(challenges::day23::Args),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Command::Day24(args) => challenges::day24::entrypoint(args),
        Command::Day23(args) => challenges::day23::entrypoint(args),
    }
}
