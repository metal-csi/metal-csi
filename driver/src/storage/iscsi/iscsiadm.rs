use super::*;
use regex::Regex;

#[derive(Debug, Deref, DerefMut, From, Into)]
pub struct Iscsiadm(ControlModule);

lazy_static! {
    static ref SESSION_LIST: Regex =
        Regex::new(r#" (?P<ip>\d+\.\d+\.\d+\.\d+):(?P<port>\d+),\d+ (?P<iqn>\S+) "#).unwrap();
}

impl Iscsiadm {
    pub fn get_target(&self, base_iqn: &str, volume_id: &str) -> String {
        let normalized_id = volume_id.replace("/", "-");
        format!("{}:{}", base_iqn, normalized_id)
    }

    pub async fn login(&self, target_name: &str, portal: &str) -> Result<()> {
        for s in self.sessions().await? {
            info!("{:?}", s);
            if s.iqn == target_name {
                return Ok(());
            }
        }
        self.exec_checked(&format!(
            "iscsiadm --mode node --targetname '{0}' --portal '{1}' --login",
            target_name, portal
        ))
        .await?;
        Ok(())
    }

    pub async fn logout(&self, target_name: &str, portal: &str) -> Result<()> {
        self.exec(&format!(
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

    pub async fn sessions(&self) -> Result<Vec<Session>> {
        let (output, code) = self.exec("iscsiadm -m session").await?;
        let mut result = vec![];
        if code == 0 {
            for cap in SESSION_LIST.captures_iter(output.as_str()) {
                result.push(Session {
                    ip: cap["ip"].to_string(),
                    port: cap["port"].to_string(),
                    iqn: cap["iqn"].to_string(),
                });
            }
        }
        info!("{} - {:?}", output, result);
        Ok(result)
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

#[derive(Debug)]
pub struct Session {
    pub ip: String,
    pub port: String,
    pub iqn: String,
}
