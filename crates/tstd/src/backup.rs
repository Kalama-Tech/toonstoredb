//! Backup and restore functionality for ToonStore

use anyhow::{Context, Result};
use chrono::Utc;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::info;

/// Backup configuration
pub struct BackupConfig {
    pub data_dir: PathBuf,
    pub backup_dir: PathBuf,
}

impl BackupConfig {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(data_dir: P1, backup_dir: Option<P2>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let backup_dir = backup_dir
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| data_dir.join("backups"));

        Self {
            data_dir,
            backup_dir,
        }
    }

    /// Create a backup of the database
    pub fn create_backup(&self, name: Option<&str>) -> Result<PathBuf> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&self.backup_dir).context("Failed to create backup directory")?;

        // Generate backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = name.unwrap_or("backup");
        let backup_filename = format!("toonstore_{}_{}.tar.gz", backup_name, timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);

        info!("Creating backup: {:?}", backup_path);

        // Create tar.gz archive
        let tar_gz = File::create(&backup_path).context("Failed to create backup file")?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        // Add all files from data directory
        let data_dir_entries =
            fs::read_dir(&self.data_dir).context("Failed to read data directory")?;

        for entry in data_dir_entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Skip backup directory itself
            if path.starts_with(&self.backup_dir) {
                continue;
            }

            // Get relative path for archive
            let relative_path = path.strip_prefix(&self.data_dir).unwrap_or(&path);

            if path.is_file() {
                info!("Adding file to backup: {:?}", relative_path);
                let mut file =
                    File::open(&path).context(format!("Failed to open file: {:?}", path))?;
                tar.append_file(relative_path, &mut file).context(format!(
                    "Failed to add file to archive: {:?}",
                    relative_path
                ))?;
            } else if path.is_dir() {
                info!("Adding directory to backup: {:?}", relative_path);
                tar.append_dir_all(relative_path, &path).context(format!(
                    "Failed to add directory to archive: {:?}",
                    relative_path
                ))?;
            }
        }

        tar.finish().context("Failed to finalize backup archive")?;

        let metadata = fs::metadata(&backup_path)?;
        info!(
            "Backup created successfully: {:?} ({} bytes)",
            backup_path,
            metadata.len()
        );

        Ok(backup_path)
    }

    /// Restore database from a backup file
    pub fn restore_backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        if !backup_path.exists() {
            anyhow::bail!("Backup file not found: {:?}", backup_path);
        }

        info!("Restoring backup from: {:?}", backup_path);

        // Create a temporary directory for extraction
        let temp_dir = self.data_dir.join(".restore_temp");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .context("Failed to clean up temporary restore directory")?;
        }
        fs::create_dir_all(&temp_dir).context("Failed to create temporary restore directory")?;

        // Extract tar.gz
        let tar_gz = File::open(backup_path).context("Failed to open backup file")?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        archive
            .unpack(&temp_dir)
            .context("Failed to extract backup archive")?;

        info!("Backup extracted to temporary directory");

        // Move current data to backup (if exists)
        let old_backup_dir = self.data_dir.join(".old_backup");
        if old_backup_dir.exists() {
            fs::remove_dir_all(&old_backup_dir).context("Failed to remove old backup directory")?;
        }

        // Move existing data files to .old_backup
        let data_entries = fs::read_dir(&self.data_dir).context("Failed to read data directory")?;

        for entry in data_entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Skip special directories
            if path == temp_dir || path == old_backup_dir || path.starts_with(&self.backup_dir) {
                continue;
            }

            // Create old_backup dir on first file
            if !old_backup_dir.exists() {
                fs::create_dir_all(&old_backup_dir)
                    .context("Failed to create old backup directory")?;
            }

            let filename = path.file_name().unwrap();
            let dest = old_backup_dir.join(filename);

            fs::rename(&path, &dest)
                .context(format!("Failed to backup existing file: {:?}", path))?;
        }

        // Move restored files to data directory
        let temp_entries = fs::read_dir(&temp_dir).context("Failed to read temporary directory")?;

        for entry in temp_entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            let filename = path.file_name().unwrap();
            let dest = self.data_dir.join(filename);

            fs::rename(&path, &dest).context(format!("Failed to restore file: {:?}", path))?;
        }

        // Clean up temporary directory
        fs::remove_dir_all(&temp_dir).context("Failed to remove temporary directory")?;

        info!("Backup restored successfully");
        info!("Previous data backed up to: {:?}", old_backup_dir);

        Ok(())
    }

    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = fs::read_dir(&self.backup_dir).context("Failed to read backup directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "gz") {
                let metadata = fs::metadata(&path)?;
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                backups.push(BackupInfo {
                    path,
                    filename,
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                });
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(backups)
    }

    /// Delete old backups, keeping only the N most recent
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize> {
        let backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let mut deleted = 0;
        for backup in backups.iter().skip(keep_count) {
            info!("Deleting old backup: {:?}", backup.path);
            fs::remove_file(&backup.path)
                .context(format!("Failed to delete backup: {:?}", backup.path))?;
            deleted += 1;
        }

        info!("Deleted {} old backup(s)", deleted);
        Ok(deleted)
    }
}

/// Information about a backup file
#[derive(Debug)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_and_restore() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().join("data");
        let backup_dir = temp.path().join("backups");

        fs::create_dir_all(&data_dir).unwrap();
        fs::write(data_dir.join("test.txt"), "test data").unwrap();

        let config = BackupConfig::new(&data_dir, Some(&backup_dir));

        // Create backup
        let backup_path = config.create_backup(Some("test")).unwrap();
        assert!(backup_path.exists());

        // Modify original file
        fs::write(data_dir.join("test.txt"), "modified data").unwrap();

        // Restore backup
        config.restore_backup(&backup_path).unwrap();

        // Verify restoration
        let content = fs::read_to_string(data_dir.join("test.txt")).unwrap();
        assert_eq!(content, "test data");
    }
}
