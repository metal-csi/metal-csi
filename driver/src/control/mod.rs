use crate::config::ControlMode;
use crate::control::local::LocalShell;
use crate::error::AppError;
use crate::util::Mount;
use crate::zfs::ZFS;
use crate::Result;
use async_trait::async_trait;
use regex::Regex;
use ssh::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

mod local;
mod ssh;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct ControlModule(Arc<Box<dyn ControlModuleTrait>>);

impl Drop for ControlModule {
    fn drop(&mut self) {
        let handle = tokio::runtime::Handle::current();
        let _enter = handle.enter();
        futures::executor::block_on(self.disconnect()).unwrap_or_default();
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct ControlStream(Box<dyn ControlStreamTrait>);

#[async_trait]
pub trait ControlModuleTrait: Send + Sync + Debug {
    async fn is_connected(&self) -> Result<bool>;
    async fn connect(&self) -> Result<()>;
    async fn exec(&self, cmd: &str) -> Result<(String, u32)>;
    async fn exec_open(&self, cmd: &str) -> Result<ControlStream>;
    async fn disconnect(&self) -> Result<()>;

    fn build_command(&self, sudo: bool, chroot: Option<&str>, cmd: &str) -> String {
        let prefix_str = if sudo { "sudo " } else { "" };
        let prefix_str = if let Some(path) = chroot {
            format!("{}chroot {} ", prefix_str, path)
        } else {
            format!("{}", prefix_str)
        };
        format!("{}{}", prefix_str, cmd)
    }

    async fn exec_checked(&self, cmd: &str) -> Result<String> {
        let (output, code) = self.exec(cmd).await?;
        if code != 0 {
            Err(AppError::CommandFailed { code, output })
        } else {
            Ok(output)
        }
    }
}

#[async_trait]
pub trait ControlStreamTrait: Send + Sync + Debug {
    async fn wait_for_completion(&mut self) -> Result<(String, u32)>;
    async fn wait_for(&mut self, ptrn: &Regex) -> Result<(String, Option<u32>)>;
    async fn sendline(&mut self, data: &str) -> Result<()>;
}

impl ControlModule {
    pub async fn get_zfs(&self) -> Result<ZFS> {
        self.connect().await?;
        Ok(self.clone().into())
    }

    pub async fn get_mount(&self) -> Result<Mount> {
        self.connect().await?;
        Ok(self.clone().into())
    }

    pub fn new(config: &ControlMode) -> Result<ControlModule> {
        match config {
            ControlMode::Local { sudo } => Ok(ControlModule(Arc::new(Box::new(LocalShell {
                sudo: *sudo,
                chroot: None,
            })))),
            ControlMode::Chroot { sudo, path } => {
                Ok(ControlModule(Arc::new(Box::new(LocalShell {
                    sudo: *sudo,
                    chroot: Some(path.to_string()),
                }))))
            }
            ControlMode::SSH {
                sudo,
                user,
                private_key,
                host,
                port,
            } => Ok(ControlModule(Arc::new(Box::new(SSHClient::new(
                user.as_str(),
                format!("{}:{}", host, port),
                private_key.to_string(),
                *sudo,
            )?)))),
        }
    }

    pub fn from_map(map: &HashMap<String, String>) -> Result<ControlModule> {
        match map.get("type").map(|v| v.as_str()) {
            Some("ssh") => {
                let user = map
                    .get("sshUser")
                    .ok_or_else(|| anyhow::anyhow!("sshUser key not found!"))?;
                let host = map
                    .get("sshHost")
                    .ok_or_else(|| anyhow::anyhow!("sshHost key not found!"))?;
                let port = map
                    .get("sshPort")
                    .ok_or_else(|| anyhow::anyhow!("sshPort key not found!"))?;
                let private_key = map
                    .get("sshKey")
                    .ok_or_else(|| anyhow::anyhow!("sshKey key not found!"))?
                    .replace("\\n", "\n");
                let sudo = map
                    .get("sudo")
                    .ok_or_else(|| anyhow::anyhow!("sudo key not found!"))?
                    .parse()?;
                Ok(ControlModule(Arc::new(Box::new(SSHClient::new(
                    user.as_str(),
                    format!("{}:{}", host, port),
                    private_key.to_string(),
                    sudo,
                )?))))
            }
            _ => Err(AppError::Generic(format!(
                "Unknown configuration type for control mode map!"
            ))),
        }
    }
}
