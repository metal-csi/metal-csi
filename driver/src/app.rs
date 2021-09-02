use crate::args::Args;
use crate::config::Configuration;
use crate::control::ControlModule;
use crate::error::Result;
use crate::zfs::ZFS;
use std::sync::Arc;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct App(Arc<InnerApp>);

#[derive(Debug)]
pub struct InnerApp {
    pub config: Configuration,
}

impl App {
    pub async fn zfs(&self) -> Result<ZFS> {
        Ok(self.control_controller().await?)
    }

    pub async fn mounts(&self) -> Result<crate::util::Mount> {
        Ok(self.control_node().await?)
    }

    pub fn new(args: Args) -> Result<Self> {
        let config = Configuration::new(args)?;
        Ok(Self(Arc::new(InnerApp { config })))
    }

    pub async fn control_controller<T: From<ControlModule>>(&self) -> Result<T> {
        let cm = ControlModule::new(&self.config.controller.control_mode)?;
        cm.connect().await?;
        Ok(cm.into())
    }

    pub async fn control_node<T: From<ControlModule>>(&self) -> Result<T> {
        let cm = ControlModule::new(&self.config.node.control_mode)?;
        cm.connect().await?;
        Ok(cm.into())
    }

    pub async fn run(&self) -> Result<()> {
        info!("Init started");
        tokio::spawn(self.clone().start_csi_services());

        let zfs: ZFS = self.control_controller().await?;
        zfs.list_datasets().await?;
        zfs.get_dataset("hoard/mongo").await?;
        drop(zfs);

        let mut iscsi = self.targetcli().await?;
        iscsi.list_iscsi_devices().await?;
        iscsi.close().await?;

        tokio::signal::ctrl_c().await?;

        info!("Shutting down");
        Ok(())
    }
}
