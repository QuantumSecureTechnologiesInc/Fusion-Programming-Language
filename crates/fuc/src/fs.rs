//! Filesystem IO for the Fusion Standard Library.
use crate::types::*;
use std::fs;

/// Represents file metadata attributes.
pub struct FileMetadata {
    pub size: FSize,
    pub is_dir: FBool,
    pub is_readonly: FBool,
}

/// Reads the entire contents of a file as a string.
pub fn read_to_string(path: &str) -> Result<FString, FString> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Writes a string to a file, creating it if necessary.
pub fn write_string(path: &str, content: &str) -> Result<(), FString> {
    fs::write(path, content).map_err(|e| e.to_string())
}

/// Checks if a file or directory exists at the given path.
pub fn exists(path: &str) -> FBool {
    std::path::Path::new(path).exists()
}

/// Retrieves metadata for a file path.
pub fn metadata(path: &str) -> Result<FileMetadata, FString> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(FileMetadata {
        size: meta.len() as FSize,
        is_dir: meta.is_dir(),
        is_readonly: meta.permissions().readonly(),
    })
}

/// Lists the entries in a directory.
pub fn read_dir(path: &str) -> Result<FVec<FString>, FString> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        entries.push(entry.path().to_string_lossy().into_owned());
    }
    Ok(entries)
}

/// Creates a directory and all of its parent directories.
pub fn create_dir_all(path: &str) -> Result<(), FString> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

/// Removes a file from the filesystem.
pub fn remove_file(path: &str) -> Result<(), FString> {
    fs::remove_file(path).map_err(|e| e.to_string())
}

/// Copies a file from one path to another.
pub fn copy(src: &str, dst: &str) -> Result<FSize, FString> {
    fs::copy(src, dst).map(|bytes| bytes as FSize).map_err(|e| e.to_string())
}

/// Renames or moves a file or directory to a new path.
pub fn rename(src: &str, dst: &str) -> Result<(), FString> {
    fs::rename(src, dst).map_err(|e| e.to_string())
}