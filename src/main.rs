use clap::Parser;

fn main() {
    let args = ssg::cli::Args::parse();

    ssg::run(args);
}
