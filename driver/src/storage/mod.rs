pub use self::filesystem::FilesystemType;
use self::iscsi::{ISCSIModule, ISCSIOptions};
use self::nfs::{NFSModule, NFSOptions};
use self::zfs::ZFSOptions;
use crate::control::ControlModule;
use crate::error::AppError;
use crate::metadata::{Metadata, Storeable};
use crate::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

mod filesystem;
mod iscsi;
mod mounter;
mod nfs;
mod zfs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageInfo {
    ISCSI {
        options: ISCSIOptions,
        zfs: ZFSOptions,
    },
    NFS {
        options: NFSOptions,
        zfs: ZFSOptions,
    },
}

impl Storeable for StorageInfo {
    const KEY: &'static str = "StorageInfo";

    fn into_bytes(self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self)?)
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(bincode::deserialize(&bytes)?)
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Storage(Arc<Box<dyn StorageModule>>);

impl Storage {
    pub async fn new_from_params_secrets(
        params: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> Result<Self> {
        let control = ControlModule::from_map(secrets)?;
        let storage_info = Self::get_storage_info_from_params(params).await?;
        Self::new_from_storage_info(storage_info, control).await
    }

    pub async fn new_from_params_secrets_metadata(
        params: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
        volume_id: &str,
        metadata: &Metadata,
    ) -> Result<Self> {
        let control = ControlModule::from_map(secrets)?;
        let storage_info = Self::get_storage_info_from_params(params).await?;
        metadata.set(volume_id, storage_info.clone()).await?;
        Self::new_from_storage_info(storage_info, control).await
    }

    pub async fn new_from_params(
        params: &HashMap<String, String>,
        control: ControlModule,
        volume_id: &str,
        metadata: &Metadata,
    ) -> Result<Self> {
        let storage_info = Self::get_storage_info_from_params(params).await?;
        metadata.set(volume_id, storage_info.clone()).await?;
        Self::new_from_storage_info(storage_info, control).await
    }

    pub async fn new_from_volume_id(
        volume_id: &str,
        control: ControlModule,
        metadata: &Metadata,
    ) -> Result<Self> {
        Self::new_from_storage_info(
            Self::get_storage_info_from_volume_id(volume_id, metadata).await?,
            control,
        )
        .await
    }

    pub async fn get_storage_info_from_volume_id(
        volume_id: &str,
        metadata: &Metadata,
    ) -> Result<StorageInfo> {
        Ok(metadata
            .get(volume_id)
            .await?
            .ok_or_else(|| AppError::Generic(format!("No metadata for specified volume ID!")))?)
    }

    pub async fn get_storage_info_from_params(
        params: &HashMap<String, String>,
    ) -> Result<StorageInfo> {
        match params.get("type").map(|s| s.as_str()) {
            Some("zfs-iscsi") => {
                let options = ISCSIOptions::new(params)?;
                let zfs = ZFSOptions::new(params)?;
                Ok(StorageInfo::ISCSI { options, zfs })
            }
            Some("zfs-nfs") => {
                let options = NFSOptions::new(params)?;
                let zfs = ZFSOptions::new(params)?;
                Ok(StorageInfo::NFS { options, zfs })
            }
            Some(s) => Err(AppError::Generic(format!(
                "'{}' is an unknown storage type!",
                s
            ))),
            None => Err(AppError::Generic(format!(
                "Storage type was not specified!"
            ))),
        }
    }

    pub async fn new_from_storage_info(
        storage_info: StorageInfo,
        control: ControlModule,
    ) -> Result<Self> {
        control.connect().await?;
        match storage_info {
            StorageInfo::ISCSI { zfs, options } => Ok(Storage(Arc::new(Box::new(ISCSIModule {
                options,
                zfs,
                control,
            })))),
            StorageInfo::NFS { zfs, options } => Ok(Storage(Arc::new(Box::new(NFSModule {
                options,
                zfs,
                control,
            })))),
        }
    }
}

#[async_trait]
pub trait StorageModule: Send + Sync + Debug {
    /// Controller creation, return type is volume_id for future requests
    async fn create(&self, name: &str, provision_size: i64) -> Result<String>;

    /// Controller deletion
    async fn delete(&self, volume_id: &str) -> Result<()>;

    /// Controller publish
    async fn publish(&self, volume_id: &str) -> Result<()>;

    /// Controller unpublish
    async fn unpublish(&self, volume_id: &str) -> Result<()>;

    /// Node stage
    async fn stage(&self, volume_id: &str, staging_path: &str) -> Result<()>;

    /// Node unstage
    async fn unstage(&self, volume_id: &str, staging_path: &str) -> Result<()>;

    /// Node publish
    async fn mount(&self, volume_id: &str, staging_path: &str, target_path: &str) -> Result<()>;

    /// Node unpublish
    async fn unmount(&self, volume_id: &str, target_path: &str) -> Result<()>;
}
