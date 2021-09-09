use crate::args::Args;
use crate::config::Configuration;
use crate::control::ControlModule;
use crate::error::Result;
use crate::zfs::ZFS;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{signal, sync::watch, time};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct App(Arc<InnerApp>);

#[derive(Debug)]
pub struct InnerApp {
    pub node_id: String,
    pub config: Configuration,
    pub csi_path: PathBuf,
    pub shutdown_tx: watch::Sender<bool>,
    pub shutdown_rx: watch::Receiver<bool>,
}

impl App {
    pub async fn zfs(&self) -> Result<ZFS> {
        Ok(self.control_controller().await?)
    }

    pub async fn mounts(&self) -> Result<crate::util::Mount> {
        Ok(self.control_node().await?)
    }

    pub fn new(args: Args) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let node_id = args.node_id.clone();
        let csi_path = args.csi_path.clone();
        let config = Configuration::new(args)?;
        Ok(Self(Arc::new(InnerApp {
            node_id,
            config,
            csi_path,
            shutdown_tx,
            shutdown_rx,
        })))
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
                Err(e) => {
                    error!("CSI driver failure! {}", e);
                    std::process::exit(2);
                }
            };
        });

        signal::ctrl_c().await?;

        self.shutdown_tx.send(true)?;
        info!("Shutdown signal sent, waiting on services to stop");
        while self.shutdown_tx.receiver_count() > 1 {
            time::sleep(time::Duration::from_millis(100)).await;
        }

        info!("Shutdown complete");
        Ok(())
    }
}
