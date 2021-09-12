use super::{
    spec::{
        node_server::Node,
        node_service_capability::{rpc, Rpc},
        *,
    },
    App,
};
use crate::{error::AppError, util::FilesystemType};
use anyhow::Result;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Node for App {
    async fn node_stage_volume(
        &self,
        request: Request<NodeStageVolumeRequest>,
    ) -> Result<Response<NodeStageVolumeResponse>, Status> {
        let message = request.get_ref();
        info!("[node] Processing stage volume request: {:?}", message);
        let vol_id = message.volume_id.as_str();

        let iscsiadm = self.control_node().await?.get_iscsiadm().await?;
        let target_name = iscsiadm.get_target(&self.config.iscsi.base_iqn, vol_id);
        iscsiadm.discovery(&self.config.iscsi.target_portal).await?;
        iscsiadm
            .login(&target_name, &self.config.iscsi.target_portal)
            .await?;
        let disk_path = iscsiadm
            .wait_for_disk(&target_name, &self.config.iscsi.target_portal)
            .await?;
        let staging_path = message.staging_target_path.as_str();

        let mounts = self.control_node().await?.get_mount().await?;
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
        let message = request.get_ref();
        info!("[node] Processing unstage volume request: {:?}", message);
        let vol_id = message.volume_id.as_str();
        let staging_path = message.staging_target_path.as_str();
        let control = self.control_node().await?;
        control.get_mount().await?.umount(&staging_path).await?;
        let iscsiadm = control.get_iscsiadm().await?;
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
        let message = request.get_ref();
        info!("[node] Processing publish volume request: {:?}", message);
        let src = message.staging_target_path.as_str();
        let dst = message.target_path.as_str();

        self.control_node()
            .await?
            .get_mount()
            .await?
            .mount(&FilesystemType::Bind, src, dst)
            .await?;

        Ok(Response::new(NodePublishVolumeResponse {}))
    }

    async fn node_unpublish_volume(
        &self,
        request: Request<NodeUnpublishVolumeRequest>,
    ) -> Result<Response<NodeUnpublishVolumeResponse>, Status> {
        let message = request.get_ref();
        info!("[node] Processing unpublish volume request: {:?}", message);
        let dst = message.target_path.as_str();
        self.control_node()
            .await?
            .get_mount()
            .await?
            .umount(dst)
            .await?;
        Ok(Response::new(NodeUnpublishVolumeResponse {}))
    }

    async fn node_get_capabilities(
        &self,
        request: Request<NodeGetCapabilitiesRequest>,
    ) -> Result<Response<NodeGetCapabilitiesResponse>, Status> {
        let message = request.get_ref();
        info!("[node] Processing get capabilities request: {:?}", message);
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
        request: Request<NodeGetInfoRequest>,
    ) -> Result<Response<NodeGetInfoResponse>, Status> {
        let message = request.get_ref();
        info!("[node] Processing get info request: {:?}", message);
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
        let message = request.get_ref();
        warn!("[node] Unhandled NodeGetVolumeStatsRequest: {:?}", message);
        Err(Status::unimplemented("Volume stats not supported!"))
    }

    async fn node_expand_volume(
        &self,
        request: Request<NodeExpandVolumeRequest>,
    ) -> Result<Response<NodeExpandVolumeResponse>, Status> {
        let message = request.get_ref();
        warn!("[node] Unhandled NodeExpandVolumeRequest: {:?}", message);
        Err(Status::unimplemented("Expand volume not supported"))
    }
}
