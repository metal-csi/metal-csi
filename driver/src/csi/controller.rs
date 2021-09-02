use super::{
    spec::{
        controller_server::Controller,
        controller_service_capability::{rpc, Rpc, Type},
        validate_volume_capabilities_response::Confirmed,
        *,
    },
    App,
};
use anyhow::Result;
use std::cmp::max;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Controller for App {
    async fn controller_get_capabilities(
        &self,
        _: Request<ControllerGetCapabilitiesRequest>,
    ) -> Result<Response<ControllerGetCapabilitiesResponse>, Status> {
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
        let name = request.get_ref().name.as_str();
        let capacity = &request.get_ref().capacity_range;
        let provision_size = if let Some(ref cap) = capacity {
            max(cap.limit_bytes, cap.required_bytes)
        } else {
            1 * 1024 * 1024 * 1024
        };

        let dataset_name = format!("{}{}", self.config.zfs.parent_dataset, name);
        let zfs = self.zfs().await?;
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
        let volume_id = request.get_ref().volume_id.as_str();
        warn!(
            "Received request to delete volume id '{}', ignored...",
            volume_id
        );
        Ok(Response::new(DeleteVolumeResponse {}))
    }

    async fn controller_publish_volume(
        &self,
        request: Request<ControllerPublishVolumeRequest>,
    ) -> Result<Response<ControllerPublishVolumeResponse>, Status> {
        let volume_id = request.get_ref().volume_id.as_str();
        // let readonly = request.get_ref().readonly; //TODO: Use this
        // let node_id = request.get_ref().node_id.as_str(); //TODO: Share to the specified node only

        let mut iscsi = self.targetcli().await?;
        let backstore = iscsi.create_backstore(volume_id).await?;

        let iqn = iscsi
            .create_target(&self.config.iscsi.base_iqn, volume_id)
            .await?;

        iscsi.set_target_backstore(&iqn, &backstore).await?;

        for (key, val) in self.config.iscsi.attributes.iter() {
            iscsi
                .set_attribute(&iqn, key.as_str(), val.as_str())
                .await?;
        }

        iscsi.close().await?;

        Ok(Response::new(ControllerPublishVolumeResponse {
            publish_context: Default::default(),
        }))
    }

    async fn controller_unpublish_volume(
        &self,
        request: Request<ControllerUnpublishVolumeRequest>,
    ) -> Result<Response<ControllerUnpublishVolumeResponse>, Status> {
        let volume_id = request.get_ref().volume_id.as_str();
        warn!(
            "Received request to unpublish volume id '{}', ignored...",
            volume_id
        );
        Ok(Response::new(ControllerUnpublishVolumeResponse {}))
    }

    async fn validate_volume_capabilities(
        &self,
        _: Request<ValidateVolumeCapabilitiesRequest>,
    ) -> Result<Response<ValidateVolumeCapabilitiesResponse>, Status> {
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
        //TODO: List volumes?
        warn!("Unhandled ListVolumesRequest: {:?}", request);
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
        //TODO: Return capacity?
        warn!("Unhandled GetCapacityRequest: {:?}", request);
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
        warn!("Unhandled CreateSnapshotResponse: {:?}", request);
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn delete_snapshot(
        &self,
        request: Request<DeleteSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        warn!("Unhandled DeleteSnapshotResponse: {:?}", request);
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn list_snapshots(
        &self,
        request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        warn!("Unhandled ListSnapshotsResponse: {:?}", request);
        Err(Status::unimplemented("Snapshots not supported!"))
    }

    async fn controller_expand_volume(
        &self,
        request: Request<ControllerExpandVolumeRequest>,
    ) -> Result<Response<ControllerExpandVolumeResponse>, Status> {
        warn!("Unhandled ControllerExpandVolumeResponse: {:?}", request);
        Err(Status::unimplemented("Expand volume not supported!"))
    }

    async fn controller_get_volume(
        &self,
        request: Request<ControllerGetVolumeRequest>,
    ) -> Result<Response<ControllerGetVolumeResponse>, Status> {
        //TODO: Retrieve volume detail
        warn!("Unhandled ControllerGetVolumeRequest: {:?}", request);
        // ControllerGetVolumeRequest { volume_id: () };
        // ControllerGetVolumeResponse {
        //     volume: (),
        //     status: (),
        // };
        Err(Status::unimplemented("Not implemented!"))
    }
}
