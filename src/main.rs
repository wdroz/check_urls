use clap::Parser;

mod checkurls;

use checkurls::get_files;

/// Simple program to greet a person

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of codebase to check
    #[clap(short, long, default_value = ".")]
    path: String,

    /// File that contains patterns to ignore
    #[clap(short, long, default_value = ".gitignore")]
    ignore_file: String,
}

#[tokio::main]
async fn main() {
    // Calling `say_world()` does not execute the body of `say_world()`.
    let args = Args::parse();
    let folder = args.path;
    get_files(folder);
}
