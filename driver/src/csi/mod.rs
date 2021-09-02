use crate::sock::UnixStream;
pub use crate::App;
use crate::Result;
use futures::TryFutureExt;
use std::path::Path;
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

        let path = "/tmp/csi.sock";
        tokio::fs::create_dir_all(Path::new(path).parent().unwrap()).await?;

        let incoming = {
            let uds = UnixListener::bind(path)?;

            async_stream::stream! {
                while let item = uds.accept().map_ok(|(st, _)| UnixStream(st)).await {
                    yield item;
                }
            }
        };

        info!("Initialized CSI services");
        Server::builder()
            .add_service(controller_service)
            .add_service(identity_service)
            .add_service(node_service)
            .serve_with_incoming(incoming)
            .await?;

        warn!("CSI Services stopped");
        Ok(())
    }
}
