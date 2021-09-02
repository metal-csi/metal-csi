use super::*;

#[derive(Debug, Deref, DerefMut, From)]
pub struct Iscsiadm(ControlModule);

impl Iscsiadm {
    pub fn get_target(&self, base_iqn: &str, volume_id: &str) -> String {
        let normalized_id = volume_id.replace("/", "-");
        format!("{}:{}", base_iqn, normalized_id)
    }

    pub async fn login(&self, target_name: &str, portal: &str) -> Result<()> {
        self.exec_checked(&format!(
            "iscsiadm --mode node --targetname '{0}' --portal '{1}' --login",
            target_name, portal
        ))
        .await?;
        Ok(())
    }

    pub async fn logout(&self, target_name: &str, portal: &str) -> Result<()> {
        self.exec_checked(&format!(
            "iscsiadm --mode node --targetname '{0}' --portal '{1}' --logout",
            target_name, portal
        ))
        .await?;
        Ok(())
    }

    pub async fn discovery(&self, portal: &str) -> Result<()> {
        self.exec_checked(&format!(
            "iscsiadm -m discovery -t sendtargets -p '{0}'",
            portal
        ))
        .await?;
        Ok(())
    }

    pub async fn wait_for_disk(&self, target_name: &str, portal: &str) -> Result<String> {
        debug!("Waiting on disk...");
        let disk_path = format!(
            "/dev/disk/by-path/ip-{}:3260-iscsi-{}-lun-0",
            portal, target_name
        );
        let cmd = format!("test -b '{}'", disk_path);
        let mut tries = 0;
        let mut code = 1;
        while code == 1 && tries < 30 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let (_, c) = self.exec(&cmd).await?;
            code = c;
            tries = tries + 1;
        }

        if code != 0 {
            Err(AppError::Generic(format!(
                "Timed out waiting for device {}",
                disk_path
            )))
        } else {
            Ok(disk_path)
        }
    }
}
