use crate::Result;
use sled::Mode;
use std::path::PathBuf;

pub trait Storeable: Sized {
    const KEY: &'static str;
    fn into_bytes(self) -> Result<Vec<u8>>;
    fn from_bytes(bytes: Vec<u8>) -> Result<Self>;
}

#[derive(Debug)]
pub struct Metadata {
    db: sled::Db,
}

impl Metadata {
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = sled::Config::default()
            .path(path)
            .mode(Mode::LowSpace)
            .cache_capacity(10_000_000) //Small DB, 10M cache is plenty
            .open()?;
        Ok(Metadata { db })
    }

    pub async fn get<T: Storeable>(&self, key: &str) -> Result<Option<T>> {
        let fullkey = format!("{}::{}", T::KEY, key);
        let raw_val = self.db.get(fullkey.as_bytes())?;
        if let Some(val) = raw_val {
            let dat: Vec<u8> = Vec::from(val.as_ref());
            if let Ok(r) = T::from_bytes(dat) {
                Ok(Some(r))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn set<T: Storeable>(&self, key: &str, val: T) -> Result<()> {
        let fullkey = format!("{}::{}", T::KEY, key);
        self.db.insert(fullkey.as_bytes(), val.into_bytes()?)?;
        self.db.flush_async().await?;
        Ok(())
    }
}
