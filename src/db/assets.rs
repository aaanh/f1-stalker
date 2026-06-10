use std::path::{Path, PathBuf};

use crate::db::store::{Database, DbError, default_assets_dir, default_db_path};

#[derive(Clone)]
pub struct AssetStore {
    db_path: PathBuf,
    assets_dir: PathBuf,
}

impl AssetStore {
    pub fn open_default() -> Result<Self, DbError> {
        let db_path = default_db_path()?;
        let assets_dir = default_assets_dir()?;
        std::fs::create_dir_all(&assets_dir)?;
        Ok(Self {
            db_path,
            assets_dir,
        })
    }

    pub fn assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    pub fn with_db<T>(&self, f: impl FnOnce(&Database) -> Result<T, DbError>) -> Result<T, DbError> {
        let db = Database::open(&self.db_path)?;
        f(&db)
    }

    pub fn load_cached(&self, url: &str) -> Result<Option<Vec<u8>>, DbError> {
        self.with_db(|db| db.load_cached_asset(url, &self.assets_dir))
    }

    pub fn is_failed(&self, url: &str) -> Result<bool, DbError> {
        self.with_db(|db| db.is_asset_failed(url, &self.assets_dir))
    }

    pub fn save_cached(&self, url: &str, bytes: &[u8]) -> Result<(), DbError> {
        self.with_db(|db| db.save_cached_asset(url, bytes, &self.assets_dir))
    }

    pub fn mark_failed(&self, url: &str) -> Result<(), DbError> {
        self.with_db(|db| db.mark_asset_failed(url))
    }

    pub fn clear(&self) -> Result<usize, DbError> {
        self.with_db(|db| db.clear_asset_cache(&self.assets_dir))
    }
}
