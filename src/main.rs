use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    Build,
    Serve,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => init(),
        Command::Build => build(),
        Command::Serve => serve(),
    }
}

fn init() {
    println!("init");
}

fn build() {
    println!("build");
}

fn serve() {
    println!("serve");
}
