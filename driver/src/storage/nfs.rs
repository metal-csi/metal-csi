use super::*;
use crate::{control::ControlModule, util::FilesystemType, zfs::ZFSOptions};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFSOptions {
    pub host: String,
}

impl NFSOptions {
    pub fn new(params: &HashMap<String, String>) -> Result<Self> {
        let host = params
            .get("host")
            .ok_or_else(|| AppError::Generic(format!("NFS Host is required!")))?
            .to_string();

        Ok(NFSOptions { host })
    }
}

#[derive(Debug)]
pub struct NFSModule {
    pub options: NFSOptions,
    pub zfs: ZFSOptions,
    pub control: ControlModule,
}

#[async_trait]
impl StorageModule for NFSModule {
    async fn create(&self, name: &str, _: i64) -> Result<String> {
        info!("Creating {}", name);
        let parent_dataset = self.zfs.parent_dataset.as_str();
        let dataset_name = format!("{}{}", parent_dataset, name);
        let zfs = self.control.get_zfs().await?;
        let dataset = zfs.get_dataset(dataset_name.as_str()).await?;
        if dataset.is_none() {
            zfs.create_dataset(dataset_name.as_str(), None).await?;
        }
        let mut attrs = self.zfs.attributes.clone();
        attrs.insert("sharenfs".into(), "on".into());
        zfs.set_attributes(&dataset_name, &attrs).await?;
        Ok(dataset_name)
    }

    async fn delete(&self, volume_id: &str) -> Result<()> {
        info!("NFS Controller Delete, no action needed: {}", volume_id);
        Ok(())
    }

    async fn publish(&self, volume_id: &str) -> Result<()> {
        info!("NFS Controller Publish, no action needed: {}", volume_id);
        Ok(())
    }

    async fn unpublish(&self, volume_id: &str) -> Result<()> {
        info!("NFS Controller Publish, no action needed: {}", volume_id);
        Ok(())
    }

    async fn stage(&self, volume_id: &str, _: &str) -> Result<()> {
        info!("NFS Node Stage, no action needed: {}", volume_id);
        Ok(())
    }

    async fn unstage(&self, volume_id: &str, _: &str) -> Result<()> {
        info!("NFS Node Unstage, no action needed: {}", volume_id);
        Ok(())
    }

    async fn mount(&self, volume_id: &str, _: &str, target_path: &str) -> Result<()> {
        info!("Mounting {}", volume_id);
        let nfs_path = format!(
            "{}:/{}",
            self.options.host, volume_id
        );
        self.control
            .get_mount()
            .await?
            .mount(&FilesystemType::NFS, &nfs_path, target_path)
            .await?;
        Ok(())
    }

    async fn unmount(&self, volume_id: &str, target_path: &str) -> Result<()> {
        info!("Unmounting {}", volume_id);
        self.control.get_mount().await?.umount(target_path).await?;
        Ok(())
    }
}
