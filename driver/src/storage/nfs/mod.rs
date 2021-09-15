use super::*;
use crate::control::ControlModule;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFSOptions {
    pub host: String,
    pub export: String,
}

impl NFSOptions {
    const EXPORT_DEFAULTS: &'static str = "wdelay,nohide,crossmnt,no_root_squash,no_subtree_check,mountpoint,sec=sys,rw,secure,no_root_squash,no_all_squash";
    const LOCAL_CIDRS: &'static str = "@192.168.0.0/16:@172.16.0.0/12:@10.0.0.0/8";

    pub fn new(params: &HashMap<String, String>) -> Result<Self> {
        let host = params
            .get("host")
            .ok_or_else(|| AppError::Generic(format!("NFS Host is required!")))?
            .to_string();

        let export = params
            .get("export")
            .map(|i| i.to_string())
            .unwrap_or_else(|| format!("{},rw={},ro", Self::EXPORT_DEFAULTS, Self::LOCAL_CIDRS));

        Ok(NFSOptions { host, export })
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
        let zfs = self.control.zfs().await?;
        let dataset = zfs.get_dataset(dataset_name.as_str()).await?;
        if dataset.is_none() {
            zfs.create_dataset(dataset_name.as_str(), None).await?;
        }
        let mut attrs = self.zfs.attributes.clone();
        attrs.insert("sharenfs".into(), self.options.export.to_string());
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
        let nfs_path = format!("{}:/{}", self.options.host, volume_id);
        self.control
            .mounter()
            .await?
            .mount(&FilesystemType::NFS, &nfs_path, target_path)
            .await?;
        Ok(())
    }

    async fn unmount(&self, volume_id: &str, target_path: &str) -> Result<()> {
        info!("Unmounting {}", volume_id);
        self.control.mounter().await?.umount(target_path).await?;
        Ok(())
    }
}
