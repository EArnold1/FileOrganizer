mod thread_pool;

use std::thread;
use std::time::Instant;
// Import standard library modules
use blake3::Hasher;
use chrono::{DateTime, Local, Utc};
use std::fs::{File, ReadDir};
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{collections::HashSet, fs, io, path::Path};
use thread_pool::WorkerPool;

use clap::Parser;
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "file_organizer", long_about = None)]
struct Args {
    #[arg(short, long, help = "Sets the file path to use")]
    path: PathBuf,

    #[arg(short, long, help = "Sets watch mode")]
    watch: bool,
}

fn main() -> io::Result<()> {
    // CLI argument parsing using clap
    let args = Args::parse();

    // Extract values from the parsed CLI arguments
    let folder_path = args.path;
    let watch_mode = args.watch;

    let start = Instant::now();
    //  Step 1: Organize all existing files once
    organize_files(&folder_path)?;

    let end = start.elapsed();

    println!("{:?}", end);

    //  Step 2: If watch mode is enabled, keep watching for new files
    if watch_mode {
        println!(
            "Watching for new files in {}",
            folder_path.to_str().unwrap()
        );
        watch_folder(&folder_path)?;
    }

    Ok(())
}

/// Organizes files by type (images, videos, etc.) and modified date
/// Also detects duplicates based on file hash
fn organize_files(folder_path: &Path) -> io::Result<()> {
    let all_files: ReadDir = fs::read_dir(folder_path)?; // Read all entries in the directory

    let mut seen_hashes = HashSet::new(); // Track hashes to detect duplicates

    let (tx, rx) = mpsc::channel();

    let num_workers = num_cpus::get();

    let pool = WorkerPool::new(num_workers);

    for entry in all_files {
        let tx = tx.clone();
        pool.execute(move || {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                thread::spawn(move || {
                    let hash = hash_file(&path).unwrap();
                    tx.send((hash, path)).expect("Should send")
                });
            }
        });
    }
    // }

    drop(tx);

    for (hash, path) in rx {
        if path.is_file() {
            //  Step 1: Hash the file to check for duplicates
            if seen_hashes.contains(&hash) {
                println!(" Duplicate found: {:?}", path.file_name().unwrap());
                move_to_folder(&path, folder_path, Some("duplicates"))?;
                continue;
            } else {
                seen_hashes.insert(hash);
            }

            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            let target_folder = match extension.as_str() {
                "jpg" | "jpeg" | "png" | "bmp" | "tiff" => "images",
                "gif" => "gifs",
                "mp4" | "mov" | "avi" | "mkv" => "videos",
                "mp3" | "wav" | "flac" => "audio",
                "pdf" | "docx" | "txt" => "documents",
                "zip" | "rar" | "7z" => "archives",
                _ => "others",
            };

            let new_path = Path::new(folder_path).join(target_folder);

            let age_category = classify_file_age(&path)?;

            move_to_folder(&path, &new_path, age_category)?;
        }
    }

    Ok(())
}

fn classify_file_age(path: &Path) -> Result<Option<&str>, io::Error> {
    let now: DateTime<Local> = Utc::now().into();
    let modified_date: DateTime<Local> = fs::metadata(path)?.modified()?.into();
    let age = now - modified_date;

    let category = match age.num_days() {
        days if days >= 60 => Some("60_days_or_older"),
        30..=59 => Some("30_to_59_days_old"),
        _ => None,
    };

    Ok(category)
}

/// Moves a file into its destination folder
/// If folder doesn’t exist, it creates it
fn move_to_folder(path: &Path, base_folder: &Path, subfolder: Option<&str>) -> io::Result<()> {
    let mut path_for_new_folder = base_folder.to_path_buf();

    if let Some(subfolder) = subfolder {
        path_for_new_folder = path_for_new_folder.join(subfolder);
    }

    if !path_for_new_folder.exists() {
        fs::create_dir_all(&path_for_new_folder)?; // Create nested directories if missing
    }

    let file_name = path.file_name().unwrap(); // Extract file name
    let new_location = path_for_new_folder.join(file_name);

    // Only move if file doesn’t already exist in destination
    if !new_location.exists() {
        fs::rename(path, &new_location)?;
        println!(" Moved {:?} → {:?}", file_name, new_location);
    }
    Ok(())
}

/// Hash the contents of a file using blake3
/// Used to detect duplicates
fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Hasher::new(); // using blake3 because it non-cryptographic, meaning it is faster compared to Sha:256
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn watch_folder(folder_path: &Path) -> io::Result<()> {
    // Create an mpsc channel to receive file system events
    let (tx, rx) = mpsc::channel();

    //  New API: Pass config instead of Duration
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).expect("Failed to initialize watcher");

    //  Convert folder_path to &Path
    watcher
        .watch(Path::new(folder_path), RecursiveMode::NonRecursive)
        .expect("Failed to start watching folder");

    println!("👀 Watching folder: {:?}", folder_path);

    // Loop to handle events
    for res in rx {
        match res {
            Ok(Event { kind, .. }) => {
                // Only trigger on file creation or modification events
                if matches!(
                    kind,
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(_))
                ) {
                    println!(" New file detected. Reorganizing...");
                    if let Err(e) = organize_files(folder_path) {
                        eprintln!(" Error during reorganization: {:?}", e);
                    }
                    // TODO: empty hash set
                }
            }
            Err(e) => println!(" Watch error: {:?}", e),
        }
    }

    Ok(())
}
