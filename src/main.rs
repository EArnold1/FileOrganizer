mod cli;
mod organizer;
mod thread_pool;

use std::time::Instant;

use clap::Parser;
use organizer::{organize_files, watcher::watch_folder};

use crate::cli::Args;

fn main() -> std::io::Result<()> {
    // CLI argument parsing using clap
    let args = Args::parse();

    // Extract values from the parsed CLI arguments
    let folder_path = args.path;
    let watch_mode = args.watch;

    let start = Instant::now();
    organize_files(&folder_path)?;
    let end = start.elapsed();

    println!("{:?}", end);

    if watch_mode {
        println!(
            "Watching for new files in {}",
            folder_path.to_str().unwrap()
        );
        watch_folder(&folder_path)?;
    }

    Ok(())
}
