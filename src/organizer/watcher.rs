use std::{io::Result, path::Path, sync::mpsc};

use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::organizer::organize_files;

pub fn watch_folder(folder_path: &Path) -> Result<()> {
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
                }
            }
            Err(e) => println!(" Watch error: {:?}", e),
        }
    }

    Ok(())
}
