use crate::config::Configuration;
use crate::control::ControlModule;
use crate::error::Result;
use crate::{args::Args, metadata::Metadata};
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
    pub csi_name: String,
    pub shutdown_tx: watch::Sender<bool>,
    pub shutdown_rx: watch::Receiver<bool>,
    pub metadata: Metadata,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let node_id = args.node_id.clone();
        let csi_path = args.csi_path.clone();
        let csi_name = args.csi_name.clone();
        let metadata = Metadata::new(args.metadata_db.clone())?;
        let config = Configuration::new(args)?;
        Ok(Self(Arc::new(InnerApp {
            node_id,
            config,
            csi_path,
            csi_name,
            shutdown_tx,
            shutdown_rx,
            metadata,
        })))
    }

    pub async fn control_node(&self) -> Result<ControlModule> {
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
