//! Storage engine implementation
//!
//! File layout:
//! - `db.toon`: Data file with TOON header + rows
//! - `db.toon.idx`: Index file mapping row IDs to offsets

use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::parser::{create_header, parse_header, TOON_IDX_MAGIC, TOON_MAGIC};

/// Maximum value size (1 MB)
const MAX_VALUE_SIZE: usize = 1024 * 1024;

/// Maximum database size (1 GB)
const MAX_DB_SIZE: u64 = 1024 * 1024 * 1024;

/// ToonStore is the main database handle
pub struct ToonStore {
    /// Path to the database directory
    #[allow(dead_code)] // Will be used for compaction
    path: PathBuf,

    /// Data file handle
    data_file: Arc<RwLock<File>>,

    /// Index file handle
    idx_file: Arc<RwLock<File>>,

    /// In-memory index: row_id -> offset in data file (None = deleted)
    index: Arc<RwLock<Vec<Option<u64>>>>,

    /// Current database size
    db_size: Arc<RwLock<u64>>,

    /// Is the database closed?
    closed: Arc<RwLock<bool>>,
}

impl ToonStore {
    /// Open or create a database at the given path
    ///
    /// # Arguments
    /// * `path` - Directory path for the database files
    ///
    /// # Returns
    /// * `Result<ToonStore>` - Database handle
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)?;

        let data_path = path.join("db.toon");
        let idx_path = path.join("db.toon.idx");

        let (data_file, idx_file, index, db_size) = if data_path.exists() {
            // Open existing database
            Self::open_existing(&data_path, &idx_path)?
        } else {
            // Create new database
            Self::create_new(&data_path, &idx_path)?
        };

        Ok(ToonStore {
            path: path.to_path_buf(),
            data_file: Arc::new(RwLock::new(data_file)),
            idx_file: Arc::new(RwLock::new(idx_file)),
            index: Arc::new(RwLock::new(index)),
            db_size: Arc::new(RwLock::new(db_size)),
            closed: Arc::new(RwLock::new(false)),
        })
    }

    fn open_existing(data_path: &Path, idx_path: &Path) -> Result<(File, File, Vec<Option<u64>>, u64)> {
        let mut data_file = OpenOptions::new().read(true).write(true).open(data_path)?;

        let mut idx_file = OpenOptions::new().read(true).write(true).open(idx_path)?;

        // Read and validate data file header
        let mut header_buf = vec![0u8; TOON_MAGIC.len() + 8];
        data_file.read_exact(&mut header_buf)?;
        let _header = parse_header(&header_buf)?;

        // Read index file
        let mut idx_magic = vec![0u8; TOON_IDX_MAGIC.len()];
        idx_file.read_exact(&mut idx_magic)?;

        if idx_magic != TOON_IDX_MAGIC {
            return Err(Error::Parse("Invalid index file magic".to_string()));
        }

        let mut count_buf = [0u8; 4];
        idx_file.read_exact(&mut count_buf)?;
        let count = u32::from_le_bytes(count_buf);

        // Read offsets
        let mut index = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut offset_buf = [0u8; 8];
            idx_file.read_exact(&mut offset_buf)?;
            let offset = u64::from_le_bytes(offset_buf);
            // Offset of 0 means deleted (we never write at offset 0 due to header)
            if offset == 0 {
                index.push(None);
            } else {
                index.push(Some(offset));
            }
        }

        // Get database size
        let db_size = data_file.seek(SeekFrom::End(0))?;

        Ok((data_file, idx_file, index, db_size))
    }

    fn create_new(data_path: &Path, idx_path: &Path) -> Result<(File, File, Vec<Option<u64>>, u64)> {
        let mut data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(data_path)?;

        let mut idx_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(idx_path)?;

        // Write TOON header
        let header = create_header(1, 0);
        data_file.write_all(&header)?;

        // Write index header
        idx_file.write_all(TOON_IDX_MAGIC)?;
        idx_file.write_all(&0u32.to_le_bytes())?; // count = 0

        let db_size = header.len() as u64;

        Ok((data_file, idx_file, Vec::new(), db_size))
    }

    /// Put a TOON line into the database
    ///
    /// # Arguments
    /// * `line` - Raw TOON line (without trailing newline)
    ///
    /// # Returns
    /// * `Result<u64>` - Row ID of the inserted line
    pub fn put(&self, line: &[u8]) -> Result<u64> {
        if *self.closed.read() {
            return Err(Error::Closed);
        }

        if line.len() > MAX_VALUE_SIZE {
            return Err(Error::ValueTooLarge(line.len()));
        }

        let mut db_size = self.db_size.write();
        if *db_size + line.len() as u64 + 1 > MAX_DB_SIZE {
            return Err(Error::DatabaseFull(*db_size));
        }

        let mut data_file = self.data_file.write();
        let mut index = self.index.write();

        // Get current offset
        let offset = data_file.seek(SeekFrom::End(0))?;

        // Write line + newline
        data_file.write_all(line)?;
        data_file.write_all(b"\n")?;

        // Update index
        let row_id = index.len() as u64;
        index.push(Some(offset));

        // Update size
        *db_size = offset + line.len() as u64 + 1;

        Ok(row_id)
    }

    /// Get a TOON line by row ID
    ///
    /// # Arguments
    /// * `row_id` - Row ID to retrieve
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Raw TOON line (without trailing newline)
    pub fn get(&self, row_id: u64) -> Result<Vec<u8>> {
        if *self.closed.read() {
            return Err(Error::Closed);
        }

        let index = self.index.read();

        if row_id >= index.len() as u64 {
            return Err(Error::NotFound);
        }

        let offset = match index[row_id as usize] {
            Some(offset) => offset,
            None => return Err(Error::NotFound), // Deleted
        };
        drop(index);

        let mut data_file = self.data_file.write();
        data_file.seek(SeekFrom::Start(offset))?;

        // Read until newline
        let mut line = Vec::new();
        let mut buf = [0u8; 1];
        loop {
            data_file.read_exact(&mut buf)?;
            if buf[0] == b'\n' {
                break;
            }
            line.push(buf[0]);
        }

        Ok(line)
    }

    /// Get the number of rows in the database
    pub fn len(&self) -> usize {
        self.index.read().len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.index.read().is_empty()
    }

    /// Delete a TOON line by row ID (soft delete - marks as deleted)
    ///
    /// # Arguments
    /// * `row_id` - Row ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Ok if deleted, Err if not found or already deleted
    pub fn delete(&self, row_id: u64) -> Result<()> {
        if *self.closed.read() {
            return Err(Error::Closed);
        }

        let mut index = self.index.write();
        
        if row_id >= index.len() as u64 {
            return Err(Error::NotFound);
        }

        if index[row_id as usize].is_none() {
            return Err(Error::NotFound); // Already deleted
        }

        // Mark as deleted
        index[row_id as usize] = None;

        Ok(())
    }

    /// Scan all non-deleted rows
    ///
    /// Returns an iterator over (row_id, line) pairs
    pub fn scan(&self) -> ScanIterator<'_> {
        ScanIterator {
            store: self,
            current: 0,
            total: self.index.read().len() as u64,
        }
    }

    /// Close the database and fsync all changes
    pub fn close(&mut self) -> Result<()> {
        if *self.closed.read() {
            return Ok(());
        }

        // Update data file header with current row count
        let index = self.index.read();
        let row_count = index.len() as u32;

        let mut data_file = self.data_file.write();
        data_file.seek(SeekFrom::Start(TOON_MAGIC.len() as u64 + 4))?;
        data_file.write_all(&row_count.to_le_bytes())?;
        data_file.sync_all()?;

        // Update index file
        let mut idx_file = self.idx_file.write();
        idx_file.seek(SeekFrom::Start(TOON_IDX_MAGIC.len() as u64))?;
        idx_file.write_all(&row_count.to_le_bytes())?;

        // Write all offsets (0 for deleted rows)
        for offset in index.iter() {
            let offset_bytes = offset.unwrap_or(0).to_le_bytes();
            idx_file.write_all(&offset_bytes)?;
        }
        idx_file.sync_all()?;

        *self.closed.write() = true;

        Ok(())
    }
}

impl Drop for ToonStore {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Iterator for scanning non-deleted rows
pub struct ScanIterator<'a> {
    store: &'a ToonStore,
    current: u64,
    total: u64,
}

impl<'a> Iterator for ScanIterator<'a> {
    type Item = Result<(u64, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.total {
            let row_id = self.current;
            self.current += 1;

            // Skip deleted rows
            let index = self.store.index.read();
            if index[row_id as usize].is_none() {
                continue;
            }
            drop(index);

            // Get the row
            match self.store.get(row_id) {
                Ok(line) => return Some(Ok((row_id, line))),
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_open() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        assert_eq!(db.len(), 0);
        assert!(db.is_empty());

        db.close().unwrap();

        // Reopen
        let db = ToonStore::open(dir.path()).unwrap();
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_put_and_get() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        let line = b"users[1]{id,name}: 1,Alice";
        let row_id = db.put(line).unwrap();

        assert_eq!(row_id, 0);
        assert_eq!(db.len(), 1);

        let retrieved = db.get(row_id).unwrap();
        assert_eq!(retrieved, line);

        db.close().unwrap();
    }

    #[test]
    fn test_multiple_puts() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        let lines = vec![
            b"users[1]{id,name}: 1,Alice".to_vec(),
            b"users[1]{id,name}: 2,Bob".to_vec(),
            b"users[1]{id,name}: 3,Charlie".to_vec(),
        ];

        for line in &lines {
            db.put(line).unwrap();
        }

        assert_eq!(db.len(), 3);

        for (i, line) in lines.iter().enumerate() {
            let retrieved = db.get(i as u64).unwrap();
            assert_eq!(&retrieved, line);
        }

        db.close().unwrap();
    }

    #[test]
    fn test_get_not_found() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        let result = db.get(0);
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_value_too_large() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        let large_line = vec![b'x'; MAX_VALUE_SIZE + 1];
        let result = db.put(&large_line);
        assert!(matches!(result, Err(Error::ValueTooLarge(_))));
    }

    #[test]
    fn test_persistence() {
        let dir = TempDir::new().unwrap();

        // Write data
        {
            let mut db = ToonStore::open(dir.path()).unwrap();
            db.put(b"test line 1").unwrap();
            db.put(b"test line 2").unwrap();
            db.close().unwrap();
        }

        // Reopen and verify
        {
            let db = ToonStore::open(dir.path()).unwrap();
            assert_eq!(db.len(), 2);
            assert_eq!(db.get(0).unwrap(), b"test line 1");
            assert_eq!(db.get(1).unwrap(), b"test line 2");
        }
    }

    #[test]
    fn test_close_twice() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        db.close().unwrap();
        db.close().unwrap(); // Should not error
    }

    #[test]
    fn test_put_after_close() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        db.close().unwrap();

        let result = db.put(b"test");
        assert!(matches!(result, Err(Error::Closed)));
    }

    #[test]
    fn test_delete() {
        let dir = TempDir::new().unwrap();
        let mut db = ToonStore::open(dir.path()).unwrap();

        let id0 = db.put(b"line 0").unwrap();
        let id1 = db.put(b"line 1").unwrap();
        let id2 = db.put(b"line 2").unwrap();

        // Delete middle row
        db.delete(id1).unwrap();

        // Can still get other rows
        assert_eq!(db.get(id0).unwrap(), b"line 0");
        assert_eq!(db.get(id2).unwrap(), b"line 2");

        // Deleted row returns NotFound
        assert!(matches!(db.get(id1), Err(Error::NotFound)));

        // Can't delete twice
        assert!(matches!(db.delete(id1), Err(Error::NotFound)));

        db.close().unwrap();
    }

    #[test]
    fn test_delete_persistence() {
        let dir = TempDir::new().unwrap();

        // Create and delete
        {
            let mut db = ToonStore::open(dir.path()).unwrap();
            db.put(b"line 0").unwrap();
            db.put(b"line 1").unwrap();
            db.put(b"line 2").unwrap();
            db.delete(1).unwrap();
            db.close().unwrap();
        }

        // Reopen and verify
        {
            let db = ToonStore::open(dir.path()).unwrap();
            assert_eq!(db.get(0).unwrap(), b"line 0");
            assert!(matches!(db.get(1), Err(Error::NotFound)));
            assert_eq!(db.get(2).unwrap(), b"line 2");
        }
    }

    #[test]
    fn test_scan() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        db.put(b"line 0").unwrap();
        db.put(b"line 1").unwrap();
        db.put(b"line 2").unwrap();

        let results: Vec<_> = db.scan().collect();
        assert_eq!(results.len(), 3);

        assert_eq!(results[0].as_ref().unwrap().0, 0);
        assert_eq!(results[0].as_ref().unwrap().1, b"line 0");
        assert_eq!(results[1].as_ref().unwrap().0, 1);
        assert_eq!(results[1].as_ref().unwrap().1, b"line 1");
        assert_eq!(results[2].as_ref().unwrap().0, 2);
        assert_eq!(results[2].as_ref().unwrap().1, b"line 2");
    }

    #[test]
    fn test_scan_with_deletes() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        db.put(b"line 0").unwrap();
        db.put(b"line 1").unwrap();
        db.put(b"line 2").unwrap();
        db.put(b"line 3").unwrap();

        // Delete some rows
        db.delete(1).unwrap();
        db.delete(3).unwrap();

        let results: Vec<_> = db.scan().collect();
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].as_ref().unwrap().0, 0);
        assert_eq!(results[0].as_ref().unwrap().1, b"line 0");
        assert_eq!(results[1].as_ref().unwrap().0, 2);
        assert_eq!(results[1].as_ref().unwrap().1, b"line 2");
    }

    #[test]
    fn test_scan_empty() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        let results: Vec<_> = db.scan().collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_delete_nonexistent() {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();

        db.put(b"line 0").unwrap();

        // Try to delete non-existent row
        assert!(matches!(db.delete(5), Err(Error::NotFound)));
    }
}
