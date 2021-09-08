use crate::sock::UnixStream;
pub use crate::App;
use crate::Result;
use futures::TryFutureExt;
use std::path::Path;
use tokio::fs;
use tokio::net::UnixListener;
use tonic::transport::Server;

mod controller;
mod identity;
mod node;

pub mod spec {
    tonic::include_proto!("csi.v1");
}

impl App {
    pub async fn start_csi_services(self) -> Result<()> {
        let controller_service = spec::controller_server::ControllerServer::new(self.clone());
        let identity_service = spec::identity_server::IdentityServer::new(self.clone());
        let node_service = spec::node_server::NodeServer::new(self.clone());

        fs::create_dir_all(Path::new(&self.csi_path).parent().unwrap()).await?;
        if self.csi_path.exists() {
            warn!("Socket already existed and had to be force deleted!");
            fs::remove_file(&self.csi_path).await?;
        }

        let incoming = {
            let uds = UnixListener::bind(&self.csi_path)?;

            async_stream::stream! {
                while let item = uds.accept().map_ok(|(st, _)| UnixStream(st)).await {
                    yield item;
                }
            }
        };

        info!("Initialized CSI services");
        let mut rx_fut = self.shutdown_rx.clone();
        let rx_drop = self.shutdown_rx.clone();

        Server::builder()
            .add_service(controller_service)
            .add_service(identity_service)
            .add_service(node_service)
            .serve_with_incoming_shutdown(incoming, async move {
                while rx_fut.changed().await.is_ok() {
                    if *rx_fut.borrow() == true {
                        break;
                    }
                }
            })
            .await?;

        warn!("CSI Services stopped");
        drop(rx_drop); //This is to ensure that the receiver count stays above baseline while the service is still shutting down
        Ok(())
    }
}
