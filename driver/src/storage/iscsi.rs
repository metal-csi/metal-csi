use super::*;
use crate::{control::ControlModule, iscsi::ISCSIOptions, util::FilesystemType};

#[derive(Debug)]
pub struct ISCSIModule {
    pub options: ISCSIOptions,
    pub zfs: ZFSOptions,
    pub control: ControlModule,
}

#[async_trait]
impl StorageModule for ISCSIModule {
    async fn create(&self, name: &str, provision_size: i64) -> Result<String> {
        info!("Creating {}", name);
        let parent_dataset = self.zfs.parent_dataset.as_str();
        let dataset_name = format!("{}{}", parent_dataset, name);
        let zfs = self.control.get_zfs().await?;
        let dataset = zfs.get_dataset(dataset_name.as_str()).await?;
        if dataset.is_none() {
            zfs.create_dataset(dataset_name.as_str(), Some(provision_size))
                .await?;
        }
        zfs.set_attributes(&dataset_name, &self.zfs.attributes)
            .await?;
        Ok(dataset_name)
    }

    async fn delete(&self, volume_id: &str) -> Result<()> {
        info!("Delete {}", volume_id);
        warn!("[iscsi] Ignoring deletion of volume '{}'", volume_id);
        Ok(())
    }

    async fn publish(&self, volume_id: &str) -> Result<()> {
        info!("Publish {}", volume_id);
        let mut targetcli = self.control.get_targetcli().await?;
        let backstore = targetcli.create_backstore(volume_id).await?;

        let base_iqn = self.options.base_iqn.as_str();

        let iqn = targetcli.create_target(base_iqn, volume_id).await?;

        targetcli.set_target_backstore(&iqn, &backstore).await?;

        for (key, val) in self.options.attributes.iter() {
            targetcli
                .set_attribute(&iqn, key.as_str(), val.as_str())
                .await?;
        }

        targetcli.close().await?;
        Ok(())
    }

    async fn unpublish(&self, volume_id: &str) -> Result<()> {
        info!("Unpublish {}", volume_id);
        warn!("[iscsi] Ignoring unpublish of volume '{}'", volume_id);
        Ok(())
    }

    async fn stage(&self, volume_id: &str, staging_path: &str) -> Result<()> {
        info!("Stage {}", volume_id);
        let iscsiadm = self.control.get_iscsiadm().await?;
        let base_iqn = self.options.base_iqn.as_str();
        let target_portal = self.options.target_portal.as_str();

        let target_name = iscsiadm.get_target(base_iqn, volume_id);
        iscsiadm.discovery(target_portal).await?;
        iscsiadm.login(&target_name, target_portal).await?;
        let disk_path = iscsiadm.wait_for_disk(&target_name, target_portal).await?;

        let mounts = self.control.get_mount().await?;
        let block_device = mounts
            .get_block_device(&disk_path)
            .await?
            .ok_or_else(|| AppError::Generic("Could not get block device detail!".into()))?;

        if let Some(fs) = block_device.fstype {
            info!("Found filesystem {} on {}", fs, &disk_path);
        } else {
            info!("Creating new filesystem on device {}", &disk_path);
            mounts.mkfs(&disk_path, &self.options.fs_type).await?;
        }

        mounts
            .mount(&FilesystemType::Ext4, &disk_path, staging_path)
            .await?;

        Ok(())
    }

    async fn unstage(&self, volume_id: &str, staging_path: &str) -> Result<()> {
        info!("Unstaging {}", volume_id);
        let control = &self.control;
        control.get_mount().await?.umount(&staging_path).await?;
        let iscsiadm = control.get_iscsiadm().await?;
        let target_name = iscsiadm.get_target(&self.options.base_iqn, volume_id);
        iscsiadm
            .logout(&target_name, &self.options.target_portal)
            .await?;
        Ok(())
    }

    async fn mount(&self, volume_id: &str, staging_path: &str, target_path: &str) -> Result<()> {
        info!("Mounting {}", volume_id);
        self.control
            .get_mount()
            .await?
            .mount(&FilesystemType::Bind, staging_path, target_path)
            .await?;
        Ok(())
    }

    async fn unmount(&self, volume_id: &str, target_path: &str) -> Result<()> {
        info!("Unmounting {}", volume_id);
        self.control.get_mount().await?.umount(target_path).await?;
        Ok(())
    }
}
