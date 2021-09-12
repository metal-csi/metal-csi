use super::{
    spec::{
        controller_server::Controller,
        controller_service_capability::{rpc, Rpc, Type},
        validate_volume_capabilities_response::Confirmed,
        *,
    },
    App,
};
use crate::{control::ControlModule, zfs::ZFS};
use anyhow::Result;
use std::cmp::max;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Controller for App {
    async fn controller_get_capabilities(
        &self,
        request: Request<ControllerGetCapabilitiesRequest>,
    ) -> Result<Response<ControllerGetCapabilitiesResponse>, Status> {
        let message = request.get_ref();
        info!(
            "[controller] Processing controller get capabilities request: {:?}",
            message
        );
        Ok(Response::new(ControllerGetCapabilitiesResponse {
            capabilities: vec![
                ControllerServiceCapability {
                    r#type: Some(Type::Rpc(Rpc {
                        r#type: rpc::Type::CreateDeleteVolume.into(),
                    })),
                },
                ControllerServiceCapability {
                    r#type: Some(Type::Rpc(Rpc {
                        r#type: rpc::Type::PublishUnpublishVolume.into(),
                    })),
                },
            ],
        }))
    }

    async fn create_volume(
        &self,
        request: Request<CreateVolumeRequest>,
    ) -> Result<Response<CreateVolumeResponse>, Status> {
        let message = request.get_ref();
        info!(
            "[controller] Processing controller create volume request: {:?}",
            message
        );

        let name = message.name.as_str();
        let capacity = &message.capacity_range;
        let provision_size = if let Some(ref cap) = capacity {
            max(cap.limit_bytes, cap.required_bytes)
        } else {
            1 * 1024 * 1024 * 1024
        };

        let dataset_name = format!("{}{}", self.config.zfs.parent_dataset, name);
        let control = ControlModule::from_map(&message.secrets)?;
        let zfs: ZFS = control.get_zfs().await?;
        let dataset = zfs.get_dataset(dataset_name.as_str()).await?;
        if dataset.is_none() {
            zfs.create_dataset(dataset_name.as_str(), Some(provision_size))
                .await?;
        }

        Ok(Response::new(CreateVolumeResponse {
            volume: Some(Volume {
                capacity_bytes: provision_size,
                volume_id: dataset_name,
                volume_context: Default::default(),
                content_source: None,
                accessible_topology: Default::default(),
            }),
        }))
    }

    async fn delete_volume(
        &self,
        request: Request<DeleteVolumeRequest>,
    ) -> Result<Response<DeleteVolumeResponse>, Status> {
        let message = request.get_ref();
        let volume_id = message.volume_id.as_str();
        warn!(
            "[controller] Received request to delete volume id '{}', ignored...",
            volume_id
        );
        Ok(Response::new(DeleteVolumeResponse {}))
    }

    async fn controller_publish_volume(
        &self,
        request: Request<ControllerPublishVolumeRequest>,
    ) -> Result<Response<ControllerPublishVolumeResponse>, Status> {
        let message = request.get_ref();
        info!(
            "[controller] Processing controller publish volume request: {:?}",
            message
        );
        let volume_id = message.volume_id.as_str();
        // let readonly = message.readonly; //TODO: Use this
        // let node_id = message.node_id.as_str(); //TODO: Share to the specified node only

        let control = ControlModule::from_map(&message.secrets)?;
        let mut targetcli = control.get_targetcli().await?;
        let backstore = targetcli.create_backstore(volume_id).await?;

        let iqn = targetcli
            .create_target(&self.config.iscsi.base_iqn, volume_id)
            .await?;

        targetcli.set_target_backstore(&iqn, &backstore).await?;

        for (key, val) in self.config.iscsi.attributes.iter() {
            targetcli
                .set_attribute(&iqn, key.as_str(), val.as_str())
                .await?;
        }

        targetcli.close().await?;

        Ok(Response::new(ControllerPublishVolumeResponse {
            publish_context: Default::default(),
        }))
    }

    async fn controller_unpublish_volume(
        &self,
        request: Request<ControllerUnpublishVolumeRequest>,
    ) -> Result<Response<ControllerUnpublishVolumeResponse>, Status> {
        let message = request.get_ref();
        let volume_id = message.volume_id.as_str();
        warn!(
            "[controller] Received request to unpublish volume id '{}', ignored...",
            volume_id
        );
        Ok(Response::new(ControllerUnpublishVolumeResponse {}))
    }

    async fn validate_volume_capabilities(
        &self,
        request: Request<ValidateVolumeCapabilitiesRequest>,
    ) -> Result<Response<ValidateVolumeCapabilitiesResponse>, Status> {
        let message = request.get_ref();
        info!(
            "[controller] Processing controller validate volume capabilities request: {:?}",
            message
        );
        Ok(Response::new(ValidateVolumeCapabilitiesResponse {
            confirmed: Some(Confirmed {
                volume_context: Default::default(),
                volume_capabilities: Default::default(),
                parameters: Default::default(),
            }),
            message: format!(""),
        }))
    }

    async fn list_volumes(
        &self,
        request: Request<ListVolumesRequest>,
    ) -> Result<Response<ListVolumesResponse>, Status> {
        let message = request.get_ref();
        //TODO: List volumes?
        warn!("[controller] Unhandled ListVolumesRequest: {:?}", message);
        // ListVolumesRequest {
        //     max_entries: (),
        //     starting_token: (),
        // };
        // ListVolumesResponse {
        //     entries: (),
        //     next_token: (),
        // };
        Err(Status::unimplemented("Not implemented!"))
    }

    async fn get_capacity(
        &self,
        request: Request<GetCapacityRequest>,
    ) -> Result<Response<GetCapacityResponse>, Status> {
        let message = request.get_ref();
        //TODO: Return capacity?
        warn!("[controller] Unhandled GetCapacityRequest: {:?}", message);
        // GetCapacityRequest {
        //     volume_capabilities: (),
        //     parameters: (),
        //     accessible_topology: (),
        // };
        // GetCapacityResponse {
        //     available_capacity: (),
        // };
        Err(Status::unimplemented("Not implemented!"))
    }

    async fn create_snapshot(
        &self,
        request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        let message = request.get_ref();
        warn!(
            "[controller] Unhandled CreateSnapshotResponse: {:?}",
            message
        );
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn delete_snapshot(
        &self,
        request: Request<DeleteSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        let message = request.get_ref();
        warn!(
            "[controller] Unhandled DeleteSnapshotResponse: {:?}",
            message
        );
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn list_snapshots(
        &self,
        request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        let message = request.get_ref();
        warn!(
            "[controller] Unhandled ListSnapshotsResponse: {:?}",
            message
        );
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn controller_expand_volume(
        &self,
        request: Request<ControllerExpandVolumeRequest>,
    ) -> Result<Response<ControllerExpandVolumeResponse>, Status> {
        let message = request.get_ref();
        warn!(
            "[controller] Unhandled ControllerExpandVolumeResponse: {:?}",
            message
        );
        Err(Status::unimplemented("Expand volume not supported!"))
    }

    async fn controller_get_volume(
        &self,
        request: Request<ControllerGetVolumeRequest>,
    ) -> Result<Response<ControllerGetVolumeResponse>, Status> {
        let message = request.get_ref();
        //TODO: Retrieve volume detail
        warn!(
            "[controller] Unhandled ControllerGetVolumeRequest: {:?}",
            message
        );
        // ControllerGetVolumeRequest { volume_id: () };
        // ControllerGetVolumeResponse {
        //     volume: (),
        //     status: (),
        // };
        Err(Status::unimplemented("Not implemented!"))
    }
}
