use std::{fs, io::Result, path::Path};

use crate::log_info;

/// Moves a file to a specified folder, creating the folder if it doesn't exist.
///
/// # Arguments
///
/// * `path` - The path to the file to be moved.
/// * `base_folder` - The base destination folder.
/// * `subfolder` - An optional subfolder within the base folder.
pub fn move_to_folder(path: &Path, base_folder: &Path, subfolder: Option<&str>) -> Result<()> {
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
        log_info!("Moved {:?} → {:?}", file_name, new_location);
    }
    Ok(())
}
