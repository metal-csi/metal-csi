use super::*;
use crate::control::ControlModule;
use crate::Result;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ISCSIOptions {
    pub base_iqn: String,
    pub target_portal: String,
    pub attributes: HashMap<String, String>,
    pub fs_type: FilesystemType,
}

impl ISCSIOptions {
    pub fn new(params: &HashMap<String, String>) -> Result<Self> {
        let base_iqn = params
            .get("baseIqn")
            .ok_or_else(|| AppError::Generic(format!("Base IQN is required!")))?
            .to_string();

        let target_portal = params
            .get("targetPortal")
            .ok_or_else(|| AppError::Generic(format!("Target Portal is required!")))?
            .to_string();

        let fs_type = params
            .get("fsType")
            .map(|fs_str| FilesystemType::from(fs_str.as_str()))
            .unwrap_or(FilesystemType::Ext4);

        let mut attributes: HashMap<String, String> = Default::default();
        for (k, v) in params.iter() {
            if k.starts_with("attr.") {
                attributes.insert(k.to_string().split_off(5), v.to_string());
            }
        }

        Ok(ISCSIOptions {
            base_iqn,
            target_portal,
            attributes,
            fs_type,
        })
    }
}

impl ControlModule {
    pub async fn get_targetcli(&self) -> Result<TargetCLI> {
        self.connect().await?;
        let targetcli = self.exec_open("targetcli").await?;
        let mut result = TargetCLI {
            cmd: self.clone(),
            targetcli,
        };
        result.wait_for_prompt().await?;
        Ok(result)
    }

    pub async fn get_iscsiadm(&self) -> Result<Iscsiadm> {
        self.connect().await?;
        Ok(self.clone().into())
    }
}
