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
        InternalServiceResponse::ConnectFailure(
            citadel_internal_service_types::ConnectFailure {
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
            if message.contains("already exists") {
                println!("Client Already Registered - Connecting");
                let connect_request = InternalServiceRequest::Connect {
                    request_id: Uuid::new_v4(),
                    username: "ClientOne".parse().unwrap(),
                    password: "secret".into(),
                    connect_mode: Default::default(),
                    udp_mode: Default::default(),
                    keep_alive_timeout: None,
                    session_security_settings: Default::default(),
                };
                service_connector.sink.send(connect_request).await.unwrap();
                let connect_response = service_connector.stream.next().await.unwrap();
                match connect_response {
                    InternalServiceResponse::ConnectSuccess(..) => {
                        println!("Client Successfully Connected to Server")
                    }
                    InternalServiceResponse::ConnectFailure(ConnectFailure { message, request_id: _ }) => {
                        panic!("Client Failed to Connect to Server: {message:?}")
                    }
                    _ => {
                        panic!("Unhandled Response While Trying to Connect to Server: {connect_response:?}")
                    }
                }
            } else {
                println!("Client Register Failed")
            }
        }
        _ => {
            panic!("Unhandled Response While Trying to Register/Connect to Server: {register_response:?}")
        }
    }
}
