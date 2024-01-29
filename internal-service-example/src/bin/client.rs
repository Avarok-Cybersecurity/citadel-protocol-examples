use citadel_internal_service::kernel::*;
use citadel_internal_service_connector::util::*;
use citadel_internal_service_types::*;
use citadel_proto::prelude::NodeType;
use citadel_proto::prelude::SessionSecuritySettingsBuilder;
use citadel_sdk::prelude::NodeBuilder;
use citadel_sdk::prelude::*;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Internal Service for Client
    let bind_address_internal_service: SocketAddr = "127.0.0.1:23457".parse().unwrap();
    let internal_service_kernel = CitadelWorkspaceService::new(bind_address_internal_service);
    let internal_service = NodeBuilder::default()
        .with_node_type(NodeType::Peer)
        .with_backend(BackendType::InMemory)
        .with_insecure_skip_cert_verification()
        .build(internal_service_kernel)
        .unwrap();
    tokio::task::spawn(internal_service);

    // Connect to Internal Service via TCP
    let mut service_connector = InternalServiceConnector::connect("127.0.0.1:23457")
        .await
        .unwrap();

    // Register To and Connect To Server
    let register_request = InternalServiceRequest::Register {
        request_id: Uuid::new_v4(),
        server_addr: "127.0.0.1:23458".parse().unwrap(),
        full_name: "Client One".parse().unwrap(),
        username: "ClientOne".parse().unwrap(),
        proposed_password: "secret".into(),
        session_security_settings: SessionSecuritySettingsBuilder::default().build().unwrap(),
        connect_after_register: true,
    };
    service_connector.sink.send(register_request).await.unwrap();
    let register_response = service_connector.stream.next().await.unwrap();
    match register_response {
        InternalServiceResponse::ConnectSuccess(
            citadel_internal_service_types::ConnectSuccess {
                cid: _,
                request_id: _,
            },
        ) => {
            println!("Client Successfully Connected to Server")
        }

        InternalServiceResponse::ConnectionFailure(
            citadel_internal_service_types::ConnectionFailure {
                message,
                request_id: _,
            },
        ) => {
            println!("Client Connection Failed: {message:?}")
        }

        InternalServiceResponse::RegisterFailure(
            citadel_internal_service_types::RegisterFailure {
                message,
                request_id: _,
            },
        ) => {
            println!("Client Register Failed: {message:?}")
        }

        _ => {
            panic!("Unhandled Response While Trying to Register/Connect to Server: {register_response:?}")
        }
    }
}
