use crate::config::ControlMode;
use crate::control::local::LocalShell;
use crate::error::AppError;
use crate::Result;
use async_trait::async_trait;
use regex::Regex;
use ssh::*;
use std::fmt::Debug;

mod local;
mod ssh;

#[derive(Debug, Deref, DerefMut)]
pub struct ControlModule(Box<dyn ControlModuleTrait>);

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
    pub fn new(config: &ControlMode) -> Result<ControlModule> {
        match config {
            ControlMode::Local { sudo } => Ok(ControlModule(Box::new(LocalShell {
                sudo: *sudo,
                chroot: None,
            }))),
            ControlMode::Chroot { sudo, path } => Ok(ControlModule(Box::new(LocalShell {
                sudo: *sudo,
                chroot: Some(path.to_string()),
            }))),
            ControlMode::SSH {
                sudo,
                user,
                private_key,
                host,
                port,
            } => Ok(ControlModule(Box::new(SSHClient::new(
                user.as_str(),
                format!("{}:{}", host, port),
                private_key.to_string(),
                *sudo,
            )?))),
        }
    }
}
