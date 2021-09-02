use super::{
    spec::{identity_server::Identity, *},
    App,
};
use anyhow::Result;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Identity for App {
    async fn get_plugin_info(
        &self,
        _: Request<GetPluginInfoRequest>,
    ) -> Result<Response<GetPluginInfoResponse>, Status> {
        info!("Plugin info requested");
        let reply = GetPluginInfoResponse {
            name: self.config.driver.name.to_string(),
            vendor_version: "0.1".into(),
            manifest: HashMap::default(),
        };
        Ok(Response::new(reply))
    }

    async fn get_plugin_capabilities(
        &self,
        _: Request<GetPluginCapabilitiesRequest>,
    ) -> Result<Response<GetPluginCapabilitiesResponse>, Status> {
        info!("Plugin capabilties requested");
        let reply = GetPluginCapabilitiesResponse {
            capabilities: vec![PluginCapability {
                r#type: Some(plugin_capability::Type::Service(
                    plugin_capability::Service {
                        r#type: plugin_capability::service::Type::ControllerService.into(),
                    },
                )),
            }],
        };
        Ok(Response::new(reply))
    }

    async fn probe(&self, _: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
        info!("Received probe...");
        let reply = ProbeResponse { ready: Some(true) };
        Ok(Response::new(reply))
    }
}
