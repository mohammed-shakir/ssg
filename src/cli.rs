use clap::{Parser, Subcommand};
use std::path::PathBuf;

// All arguments
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

// All available actions
#[derive(Subcommand, Debug)]
pub enum Action {
    Build {
        #[arg(short, long, default_value = "src")]
        src: PathBuf,

        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    Serve {
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    Clean {
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
}
