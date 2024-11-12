use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::extraction::pdf::{ExtractedTables, TableReader};

#[cfg(test)]
mod tests;

const MAX_FILE_HASH_LRU: usize = 100;
const MAX_TABLES_LRU: usize = 1000;
const PERSIST_CACHE_VERSION: &str = "1";

pub struct CachingTableReader<T> {
    delegate: T,
    tables_cache_path: PathBuf,
    tables_cache: TablesCache,
    file_hashes_cache: FileHashesCache,
}

impl<T> CachingTableReader<T> {
    pub fn load(delegate: T, tables_cache_path: PathBuf) -> Result<Self> {
        let tables_cache = TablesCache::new();
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
                log::warn!("Failed to read existing tables cache: {err}");
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
    fn read_pdf_with_template(
        &self,
        pdf_path: &Path,
        template_json: &str,
    ) -> anyhow::Result<ExtractedTables> {
        let pdf_hash = self.hash_file(pdf_path)?;
        let key = TablesCache::key(&pdf_hash, template_json)?;

        if let Some(tables) = self.tables_cache.get(&key) {
            // Cache hit.
            return Ok(tables);
        }

        // Cache miss
        let tables = self
            .delegate
            .read_pdf_with_template(pdf_path, template_json)?;

        self.tables_cache.put(key, tables.clone());

        Ok(tables)
    }

    fn close(self: Box<Self>) -> Result<()> {
        self.store()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct PersistantCache {
    version: String,
    entries: HashMap<HashDigest, ExtractedTables>,
}

struct TablesCache {
    tables_cache: Mutex<lru::LruCache<HashDigest, ExtractedTables>>,
}

impl TablesCache {
    fn new() -> Self {
        Self {
            tables_cache: Mutex::new(lru::LruCache::new(
                MAX_TABLES_LRU.try_into().expect("should never be zero"),
            )),
        }
    }

    fn key(pdf_hash: &HashDigest, template_json: &str) -> Result<HashDigest> {
        let mut hash = HashAlgo::default();
        std::hash::Hasher::write(&mut hash, &pdf_hash.0);
        std::hash::Hasher::write(&mut hash, template_json.as_bytes());
        hash_digest(&mut hash).with_context(|| "generating PDF+template hash")
    }

    fn len(&self) -> usize {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for len")
            .len()
    }

    fn load(&self, entries: impl Iterator<Item = (HashDigest, ExtractedTables)>) {
        let mut guard = self
            .tables_cache
            .lock()
            .expect("failed to lock tables_cache for load_tables");
        for entry in entries {
            guard.put(entry.0, entry.1);
        }
    }

    fn dump(self) -> HashMap<HashDigest, ExtractedTables> {
        self.tables_cache
            .into_inner()
            .expect("failed to lock tables_cache for dump_tables")
            .into_iter()
            .collect()
    }

    fn get(&self, hash: &HashDigest) -> Option<ExtractedTables> {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for get_tables")
            .get(hash)
            .cloned()
    }

    fn put(&self, hash: HashDigest, tables: ExtractedTables) {
        self.tables_cache
            .lock()
            .expect("failed to lock tables_cache for put_tables")
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

type HashAlgo = sha::sha256::Sha256;

const HASH_DIGEST_LEN: usize = 32;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
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
