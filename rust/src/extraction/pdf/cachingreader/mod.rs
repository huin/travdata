use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result};
use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};

use super::{ExtractedTable, TableReader};
use crate::extraction::pdf::template;

#[cfg(test)]
mod tests;

const MAX_FILE_HASH_LRU: usize = 100;
const MAX_TABLES_LRU: usize = 1000;
const PERSIST_CACHE_VERSION: &str = "2";

pub struct CachingTableReader<T> {
    delegate: T,
    tables_cache_path: PathBuf,
    tables_cache: TableCache,
    file_hashes_cache: FileHashesCache,
}

impl<T> CachingTableReader<T> {
    pub fn load(delegate: T, tables_cache_path: PathBuf) -> Result<Self> {
        let tables_cache = TableCache::new();
        match Self::read_cache_file(&tables_cache_path) {
            Ok(Some(loaded_cache)) => {
                tables_cache.load(loaded_cache.entries.into_iter());
                let num_entries = tables_cache.len();
                log::debug!("Loaded {num_entries} entries from existing tables cache.");
            }
            Ok(None) => {
                log::info!("Did not find existing tables cache.");
            }
            Err(err) => {
                log::warn!("Failed to read existing table cache: {err}");
            }
        };

        Ok(Self {
            delegate,
            tables_cache_path,
            tables_cache,
            file_hashes_cache: FileHashesCache::new(),
        })
    }

    fn read_cache_file(cache_path: &Path) -> Result<Option<PersistantCache>> {
        let cache_file = match File::open(cache_path) {
            Ok(cache_file) => cache_file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(err) => {
                Err(err).with_context(|| format!("opening file {:?} to read cache", cache_path))?
            }
        };
        let cache: PersistantCache = serde_json::from_reader(cache_file)
            .with_context(|| format!("reading cache as JSON from file {:?}", cache_path))?;
        if cache.version != PERSIST_CACHE_VERSION {
            return Ok(None);
        }
        Ok(Some(cache))
    }

    pub fn store(self) -> Result<()> {
        let dumped_cache = PersistantCache {
            version: PERSIST_CACHE_VERSION.to_owned(),
            entries: self.tables_cache.dump(),
        };

        let cache_file = File::create(&self.tables_cache_path)
            .with_context(|| format!("opening file {:?} to write cache", self.tables_cache_path))?;
        serde_json::to_writer(cache_file, &dumped_cache).with_context(|| {
            format!("writing cache as JSON to file {:?}", self.tables_cache_path)
        })?;

        Ok(())
    }

    fn hash_file(&self, path: &Path) -> Result<HashDigest> {
        let canonical_path = std::fs::canonicalize(path)
            .with_context(|| format!("resolving canonical path for file {:?}", path))?;

        let stat = std::fs::metadata(&canonical_path)
            .with_context(|| format!("reading metadata for file {:?}", canonical_path))?;
        let size = stat.len();
        let mtime = stat
            .modified()
            .with_context(|| format!("reading modified time for file {:?}", canonical_path))?
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .with_context(|| format!("resolving UNIX timestamp for file {:?}", canonical_path))?;

        if let Some(hash_entry) = self.file_hashes_cache.get(&canonical_path) {
            // Cache hit.
            if hash_entry.st_mtime == mtime && hash_entry.st_size == size {
                // Cache entry likely valid.
                return Ok(hash_entry.digest);
            }
        }

        // Cache miss.

        let mut f = std::fs::File::open(path)
            .with_context(|| format!("opening file {:?} for hashing", path))?;
        let mut hash = HashAlgo::default();
        std::io::copy(&mut f, &mut hash)
            .with_context(|| format!("reading file {:?} for hashing", path))?;
        let digest = hash_digest(&mut hash).with_context(|| "generating PDF content hash")?;

        self.file_hashes_cache.put(
            path.to_owned(),
            FileHashEntry {
                digest,
                st_size: size,
                st_mtime: mtime,
            },
        );

        Ok(digest)
    }
}

impl<T> TableReader for CachingTableReader<T>
where
    T: TableReader,
{
    fn read_table_portion(
        &self,
        pdf_path: &Path,
        table_portion: &template::TablePortion,
    ) -> Result<ExtractedTable> {
        let pdf_hash = self.hash_file(pdf_path)?;

        let hash_src = TablePortionHashSource {
            pdf_hash,
            extraction_method: table_portion.extraction_method,
            page: table_portion.page,
            left: table_portion.rect.left.to_f32(),
            top: table_portion.rect.top.to_f32(),
            right: table_portion.rect.right.to_f32(),
            bottom: table_portion.rect.bottom.to_f32(),
        };

        let key = TableCache::key(&hash_src)?;

        match self.tables_cache.get(&key) {
            Some(table) => {
                // Cache hit.
                Ok(table)
            }
            None => {
                // Cache miss
                let table = self.delegate.read_table_portion(pdf_path, table_portion)?;
                self.tables_cache.put(key, table.clone());
                Ok(table)
            }
        }
    }

    fn close(self: Box<Self>) -> Result<()> {
        self.store()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct PersistantCache {
    version: String,
    entries: HashMap<HashDigest, ExtractedTable>,
}

struct TableCache {
    tables_cache: Mutex<lru::LruCache<HashDigest, ExtractedTable>>,
}

impl TableCache {
    fn new() -> Self {
        Self {
            tables_cache: Mutex::new(lru::LruCache::new(
                MAX_TABLES_LRU.try_into().expect("should never be zero"),
            )),
        }
    }

    fn key(table_hash_src: &TablePortionHashSource) -> Result<HashDigest> {
        let mut hash = HashAlgo::default();

        table_hash_src
            .serialize(&mut hash)
            .with_context(|| "hashing table portion arguments")?;

        hash_digest(&mut hash).with_context(|| "generating hash digest")
    }

    fn len(&self) -> usize {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for len")
            .len()
    }

    fn load(&self, entries: impl Iterator<Item = (HashDigest, ExtractedTable)>) {
        let mut guard = self
            .tables_cache
            .lock()
            .expect("failed to lock tables_cache for load");
        for entry in entries {
            guard.put(entry.0, entry.1);
        }
    }

    fn dump(self) -> HashMap<HashDigest, ExtractedTable> {
        self.tables_cache
            .into_inner()
            .expect("failed to lock tables_cache for dump")
            .into_iter()
            .collect()
    }

    fn get(&self, hash: &HashDigest) -> Option<ExtractedTable> {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for get")
            .get(hash)
            .cloned()
    }

    fn put(&self, hash: HashDigest, tables: ExtractedTable) {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for put")
            .put(hash, tables);
    }
}

#[derive(Clone, Copy)]
struct FileHashEntry {
    digest: HashDigest,
    st_size: u64,
    st_mtime: std::time::Duration,
}

struct FileHashesCache {
    file_hash_cache: Mutex<lru::LruCache<PathBuf, FileHashEntry>>,
}

impl FileHashesCache {
    fn new() -> Self {
        Self {
            file_hash_cache: Mutex::new(lru::LruCache::new(
                MAX_FILE_HASH_LRU.try_into().expect("should never be zero"),
            )),
        }
    }

    fn get(&self, canonical_path: &Path) -> Option<FileHashEntry> {
        self.file_hash_cache
            .lock()
            .expect("failed to lock file_hash_cache for get")
            .get(canonical_path)
            .copied()
    }

    fn put(&self, canonical_path: PathBuf, entry: FileHashEntry) {
        self.file_hash_cache
            .lock()
            .expect("failed to lock file_hash_cache for put")
            .put(canonical_path, entry);
    }
}

/// Internal struct used to generate a hash for a cache key for a table portion within a specific
/// PDF file.
#[derive(BorshSerialize)]
struct TablePortionHashSource {
    pdf_hash: HashDigest,
    extraction_method: template::TabulaExtractionMethod,
    page: i32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

type HashAlgo = sha::sha256::Sha256;

const HASH_DIGEST_LEN: usize = 32;

#[derive(
    BorshSerialize, Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Deserialize, Serialize,
)]
#[serde(try_from = "String", into = "OwnString")]
struct HashDigest([u8; HASH_DIGEST_LEN]);

impl TryFrom<String> for HashDigest {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::prelude::v1::Result<Self, Self::Error> {
        let mut digest = HashDigest::default();
        hex::decode_to_slice(value, &mut digest.0)?;
        Ok(digest)
    }
}

#[derive(Serialize)]
struct OwnString(String);

impl From<HashDigest> for OwnString {
    fn from(value: HashDigest) -> Self {
        OwnString(hex::encode(value.0))
    }
}

fn hash_digest(hash: &mut HashAlgo) -> Result<HashDigest> {
    hash.flush().with_context(|| "flushing hash")?;
    let digest_vec = sha::utils::DigestExt::to_bytes(hash);
    let mut digest = HashDigest::default();
    digest.0.copy_from_slice(&digest_vec);
    Ok(digest)
}
