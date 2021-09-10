use crate::{error::AppError, util::FilesystemType};

use super::{
    spec::{
        node_server::Node,
        node_service_capability::{rpc, Rpc},
        *,
    },
    App,
};
use anyhow::Result;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Node for App {
    async fn node_stage_volume(
        &self,
        request: Request<NodeStageVolumeRequest>,
    ) -> Result<Response<NodeStageVolumeResponse>, Status> {
        info!("[node] Processing stage volume request: {:?}", request);
        let req = request.get_ref();
        let vol_id = req.volume_id.as_str();

        let iscsiadm = self.iscsiadm().await?;
        let target_name = iscsiadm.get_target(&self.config.iscsi.base_iqn, vol_id);
        iscsiadm.discovery(&self.config.iscsi.target_portal).await?;
        iscsiadm
            .login(&target_name, &self.config.iscsi.target_portal)
            .await?;
        let disk_path = iscsiadm
            .wait_for_disk(&target_name, &self.config.iscsi.target_portal)
            .await?;
        let staging_path = req.staging_target_path.as_str();

        let mounts = self.mounts().await?;
        let block_device = mounts
            .get_block_device(&disk_path)
            .await?
            .ok_or_else(|| AppError::Generic("Could not get block device detail!".into()))?;

        if let Some(fs) = block_device.fstype {
            info!("Found filesystem {} on {}", fs, &disk_path);
        } else {
            info!("Creating new filesystem on device {}", &disk_path);
            mounts.mkfs(&disk_path, &FilesystemType::Ext4).await?;
        }

        mounts
            .mount(&FilesystemType::Ext4, &disk_path, staging_path)
            .await?;

        Ok(Response::new(NodeStageVolumeResponse {}))
    }

    async fn node_unstage_volume(
        &self,
        request: Request<NodeUnstageVolumeRequest>,
    ) -> Result<Response<NodeUnstageVolumeResponse>, Status> {
        info!("[node] Processing unstage volume request: {:?}", request);
        let req = request.get_ref();
        let vol_id = req.volume_id.as_str();
        let staging_path = req.staging_target_path.as_str();
        self.mounts().await?.umount(&staging_path).await?;
        let iscsiadm = self.iscsiadm().await?;
        let target_name = iscsiadm.get_target(&self.config.iscsi.base_iqn, vol_id);
        iscsiadm
            .logout(&target_name, &self.config.iscsi.target_portal)
            .await?;
        Ok(Response::new(NodeUnstageVolumeResponse {}))
    }

    async fn node_publish_volume(
        &self,
        request: Request<NodePublishVolumeRequest>,
    ) -> Result<Response<NodePublishVolumeResponse>, Status> {
        info!("[node] Processing publish volume request: {:?}", request);
        let req = request.get_ref();
        let src = req.staging_target_path.as_str();
        let dst = req.target_path.as_str();

        self.mounts()
            .await?
            .mount(&FilesystemType::Bind, src, dst)
            .await?;

        Ok(Response::new(NodePublishVolumeResponse {}))
    }

    async fn node_unpublish_volume(
        &self,
        request: Request<NodeUnpublishVolumeRequest>,
    ) -> Result<Response<NodeUnpublishVolumeResponse>, Status> {
        info!("[node] Processing unpublish volume request: {:?}", request);
        let req = request.get_ref();
        let dst = req.target_path.as_str();
        self.mounts().await?.umount(dst).await?;
        Ok(Response::new(NodeUnpublishVolumeResponse {}))
    }

    async fn node_get_capabilities(
        &self,
        _: Request<NodeGetCapabilitiesRequest>,
    ) -> Result<Response<NodeGetCapabilitiesResponse>, Status> {
        info!("[node] Processing get capabilities request: {:?}", request);
        let reply = NodeGetCapabilitiesResponse {
            capabilities: vec![NodeServiceCapability {
                r#type: Some(node_service_capability::Type::Rpc(Rpc {
                    r#type: rpc::Type::StageUnstageVolume.into(),
                })),
            }],
        };
        Ok(Response::new(reply))
    }

    async fn node_get_info(
        &self,
        _: Request<NodeGetInfoRequest>,
    ) -> Result<Response<NodeGetInfoResponse>, Status> {
        info!("[node] Processing get info request: {:?}", request);
        let reply = NodeGetInfoResponse {
            node_id: self.node_id.to_string(),
            max_volumes_per_node: 0,
            accessible_topology: None,
        };
        Ok(Response::new(reply))
    }

    async fn node_get_volume_stats(
        &self,
        request: Request<NodeGetVolumeStatsRequest>,
    ) -> Result<Response<NodeGetVolumeStatsResponse>, Status> {
        warn!("[node] Unhandled NodeGetVolumeStatsResponse: {:?}", request);
        Err(Status::unimplemented("Volume stats not supported!"))
    }

    async fn node_expand_volume(
        &self,
        request: Request<NodeExpandVolumeRequest>,
    ) -> Result<Response<NodeExpandVolumeResponse>, Status> {
        warn!("[node] Unhandled NodeExpandVolumeResponse: {:?}", request);
        Err(Status::unimplemented("Expand volume not supported"))
    }
}
