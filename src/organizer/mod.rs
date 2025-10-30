use std::{collections::HashSet, fs, io::Result, path::Path, sync::mpsc};

use crate::{log_warn, thread_pool::WorkerPool};
use file_ops::move_to_folder;

pub mod file_ops;
pub mod watcher;

mod hasher {
    use std::{fs::File, io::prelude::*, io::Result, path::Path};

    use blake3::Hasher;

    /// Hash the contents of a file using blake3
    /// Used to detect duplicates
    pub fn hash_file(path: &Path) -> Result<String> {
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
}

mod classifier {
    use std::{fs, io::Result, path::Path};

    use chrono::{DateTime, Local};

    pub fn classify_file_age(path: &Path) -> Result<Option<&str>> {
        let now = Local::now();
        let modified_date: DateTime<Local> = fs::metadata(path)?.modified()?.into();
        let age = now - modified_date;

        let category = match age.num_days() {
            (8_i64..=30_i64) => Some("Previous_30_days"),
            (31..60) => Some("Previous_60_days"),
            _ => None,
        };

        Ok(category)
    }
}

fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name_str| name_str.starts_with('.'))
}

/// Organizes files by type (images, videos, etc.) and modified date
/// Also detects duplicates based on file hash
pub fn organize_files(folder_path: &Path) -> Result<()> {
    let all_files = fs::read_dir(folder_path)?; // Read all entries in the directory

    let mut seen_hashes = HashSet::new(); // Track hashes to detect duplicates

    let (tx, rx) = mpsc::channel();

    let num_workers = num_cpus::get();

    let pool = WorkerPool::new(num_workers);

    for entry in all_files {
        let entry = entry?;
        let path = entry.path();

        // skip hidden files
        if is_hidden_file(&path) {
            continue;
        }

        let tx = tx.clone();
        pool.execute(move || {
            if path.is_file() {
                std::thread::spawn(move || {
                    let hash = hasher::hash_file(&path).unwrap();
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
                log_warn!(" Duplicate found: {:?}", path.file_name().unwrap());
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

            let new_path = std::path::Path::new(folder_path).join(target_folder);

            let age_category = classifier::classify_file_age(&path)?;

            move_to_folder(&path, &new_path, age_category)?;
        }
    }

    Ok(())
}
