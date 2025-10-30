use std::{io::Result, path::Path, sync::mpsc};

use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::{log_error, log_info, organizer::organize_files};

/// Watches a folder for new files and triggers the organization process.
///
/// # Arguments
///
/// * `folder_path` - The path to the folder to be monitored.
pub fn watch_folder(folder_path: &Path) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).expect("Failed to initialize watcher");

    watcher
        .watch(folder_path, RecursiveMode::NonRecursive)
        .expect("Failed to start watching folder");

    log_info!("Watching folder: {:?}", folder_path);

    // Loop to handle events
    for res in rx {
        match res {
            Ok(Event { kind, .. }) => {
                // Only trigger on file creation or modification events
                if matches!(
                    kind,
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(_))
                ) {
                    log_info!("New file detected. Reorganizing...");
                    if let Err(e) = organize_files(folder_path) {
                        log_error!(" Error during reorganization: {:?}", e);
                    }
                }
            }
            Err(e) => log_error!("Watcher error: {:?}", e),
        }
    }

    Ok(())
}
