use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "file_organizer", long_about = None)]
pub struct Args {
    #[arg(short, long, help = "Sets the file path to use")]
    pub path: PathBuf,

    #[arg(short, long, help = "Sets watch mode")]
    pub watch: bool,
}
