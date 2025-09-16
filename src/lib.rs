pub mod cli;
pub mod config;
pub mod content;
pub mod render;

use cli::{Action, Args};
use std::path::Path;

pub fn run(args: Args) {
    match args.action {
        Action::Build { src, out } => build(&src, &out),
        Action::Serve { out } => serve(&out),
        Action::Clean { out } => clean(&out),
    }
}

fn build(src: &Path, out: &Path) {
    println!("Build from {:?} to {:?}", src, out);
}

fn serve(out: &Path) {
    println!("Serve from {:?}", out);
}

fn clean(out: &Path) {
    println!("Clean {:?}", out);
}
