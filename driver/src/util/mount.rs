use std::fmt::Write;

use super::FilesystemType;
use crate::control::ControlModule;
use crate::error::AppError;
use crate::Result;

#[derive(Debug, Deref, DerefMut, From)]
pub struct Mount(ControlModule);

#[allow(dead_code)]
impl Mount {
    pub async fn mount(&self, fs: &FilesystemType, device: &str, path: &str) -> Result<()> {
        self.exec_checked(&format!("mkdir -p {}", path)).await?;
        info!("Mounting device {} at path {}", device, path);
        let mut cmd = format!("mount ");
        if let Some(s) = fs.mount_type() {
            cmd.write_fmt(format_args!("-t {} ", s))?;
        }
        if let Some(o) = fs.mount_options() {
            cmd.write_fmt(format_args!("-o {} ", o))?;
        }
        cmd.write_fmt(format_args!("'{}' '{}'", device, path))?;

        debug!("Running mount command: {}", cmd);
        let (output, code) = self.exec(&cmd).await?;
        if code == 0 {
            return Ok(());
        } else if code == 32 {
            if output.contains("already mounted") {
                return Ok(());
            }
        }
        Err(AppError::CommandFailed { code, output })
    }

    pub async fn umount(&self, path: &str) -> Result<()> {
        info!("Unmounting {}", path);
        let cmd = format!("umount '{}'", path);
        let (output, code) = self.exec(&cmd).await?;
        if code == 0 {
            return Ok(());
        } else if code == 32 {
            if output.contains("not mounted") {
                return Ok(());
            }
        }
        Err(AppError::CommandFailed { code, output })
    }

    pub async fn get_mount(&self, path: &str) -> Result<Option<MountDetail>> {
        let result = self
            .exec_checked(&format!(
                "findmnt -J -o '{}' {}",
                MountDetail::COLUMNS,
                path
            ))
            .await;
        match result {
            Ok(output) => {
                let mut mdc: MountDetailContainer = serde_json::from_str(&output)?;
                Ok(mdc.filesystems.pop())
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn get_mounts(&self) -> Result<Vec<MountDetail>> {
        let result = self
            .exec_checked(&format!("findmnt -J -o '{}'", MountDetail::COLUMNS))
            .await?;
        let mdc: MountDetailContainer = serde_json::from_str(&result)?;
        Ok(mdc.filesystems)
    }

    pub async fn get_block_device(&self, path: &str) -> Result<Option<BlockDevice>> {
        let result = self
            .exec_checked(&format!(
                "lsblk -J -o '{}' '{}'",
                BlockDevice::COLUMNS,
                path
            ))
            .await?;
        let mut bdc: BlockDeviceContainer = serde_json::from_str(&result)?;
        Ok(bdc.blockdevices.pop())
    }

    pub async fn mkfs(&self, path: &str, fs: &FilesystemType) -> Result<()> {
        info!("Creating a {} filesystem on {}", fs, path);
        let cmd = format!(
            "{} '{}'",
            fs.mkfs().ok_or_else(|| AppError::Generic(format!(
                "Cannot make filesystem for {}",
                fs.mount_type().unwrap_or_default()
            )))?,
            path
        );
        self.exec_checked(&cmd).await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountDetailContainer {
    filesystems: Vec<MountDetail>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MountDetail {
    pub id: i64,
    pub source: String,
    pub target: String,
    pub fstype: Option<String>,
    pub label: Option<String>,
    pub options: Option<String>,
    pub avail: Option<String>,
    pub size: Option<String>,
    pub used: Option<String>,
    pub partuuid: Option<String>,
    pub children: Vec<MountDetail>,
}

impl MountDetail {
    const COLUMNS: &'static str = "id,source,target,fstype,label,options,partuuid,avail,size,used";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDeviceContainer {
    blockdevices: Vec<BlockDevice>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BlockDevice {
    pub name: String,
    pub rm: Bool,
    pub r#type: String,
    pub size: String,
    pub fstype: Option<String>,
    pub ro: Bool,
}

impl BlockDevice {
    const COLUMNS: &'static str = "name,rm,type,size,fstype,ro";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bool {
    Bool(bool),
    String(String),
}

impl Default for Bool {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl Bool {
    pub fn val(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::String(s) => match s.as_str() {
                "1" | "true" | "True" | "TRUE" => true,
                _ => false,
            },
        }
    }
}
