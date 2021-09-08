use crate::args::Args;
use crate::config::Configuration;
use crate::control::ControlModule;
use crate::error::Result;
use crate::zfs::ZFS;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct App(Arc<InnerApp>);

#[derive(Debug)]
pub struct InnerApp {
    pub config: Configuration,
    pub csi_path: PathBuf,
}

impl App {
    pub async fn zfs(&self) -> Result<ZFS> {
        Ok(self.control_controller().await?)
    }

    pub async fn mounts(&self) -> Result<crate::util::Mount> {
        Ok(self.control_node().await?)
    }

    pub fn new(args: Args) -> Result<Self> {
        let csi_path = args.csi_path.clone();
        let config = Configuration::new(args)?;
        Ok(Self(Arc::new(InnerApp { config, csi_path })))
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

        let zelf = self.clone();
        tokio::spawn(async move {
            info!("Spawning CSI task");
            match zelf.start_csi_services().await {
                Ok(_) => {}
                Err(e) => warn!("CSI driver failure! {}", e),
            };
        });

        tokio::signal::ctrl_c().await?;

        info!("Shutting down");
        Ok(())
    }
}
