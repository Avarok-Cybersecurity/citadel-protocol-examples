use citadel_internal_service::kernel::*;
use citadel_internal_service_connector::util::*;
use citadel_internal_service_types::*;
use citadel_proto::prelude::NodeType;
use citadel_proto::prelude::SessionSecuritySettingsBuilder;
use citadel_sdk::prelude::NodeBuilder;
use citadel_sdk::prelude::*;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::time::Duration;
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
        full_name: "Client Two".parse().unwrap(),
        username: "ClientTwo".parse().unwrap(),
        proposed_password: "secret".into(),
        session_security_settings: SessionSecuritySettingsBuilder::default().build().unwrap(),
        connect_after_register: true,
    };
    service_connector.sink.send(register_request).await.unwrap();
    let register_response = service_connector.stream.next().await.unwrap();
    let cid = match register_response {
        InternalServiceResponse::ConnectSuccess(
            citadel_internal_service_types::ConnectSuccess { cid, request_id: _ },
        ) => {
            println!("Client Successfully Connected to Server");
            cid
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
                    username: "ClientTwo".parse().unwrap(),
                    password: "secret".into(),
                    connect_mode: Default::default(),
                    udp_mode: Default::default(),
                    keep_alive_timeout: None,
                    session_security_settings: Default::default(),
                };
                service_connector.sink.send(connect_request).await.unwrap();
                let connect_response = service_connector.stream.next().await.unwrap();
                match connect_response {
                    InternalServiceResponse::ConnectSuccess(
                        citadel_internal_service_types::ConnectSuccess { cid, request_id: _ },
                    ) => {
                        println!("Client Successfully Connected to Server");
                        cid
                    }
                    InternalServiceResponse::ConnectFailure(..) => {
                        panic!("Client Failed to Connect to Server")
                    }
                    _ => {
                        panic!("Unhandled Response While Trying to Connect to Server: {connect_response:?}")
                    }
                }
            } else {
                panic!("Client Register Failed")
            }
        }
        _ => {
            panic!("Unhandled Response While Trying to Register/Connect to Server: {register_response:?}")
        }
    };

    // Receive Register Request from Peer One
    let inbound_register_request = service_connector.stream.next().await.unwrap();
    let peer_cid = match inbound_register_request {
        InternalServiceResponse::PeerRegisterNotification(PeerRegisterNotification {
            cid: _,
            peer_cid,
            peer_username,
            request_id: _,
        }) => {
            println!("Received Register Request from {peer_username:?}");
            peer_cid
        }
        _ => {
            panic!("Unexpected Response {inbound_register_request:?}")
        }
    };

    // Accept Register Request from Peer One
    let peer_register_request = InternalServiceRequest::PeerRegister {
        request_id: Uuid::new_v4(),
        cid,
        peer_cid,
        session_security_settings: Default::default(),
        connect_after_register: false,
    };
    service_connector
        .sink
        .send(peer_register_request)
        .await
        .unwrap();
    let peer_register_response = service_connector.stream.next().await.unwrap();
    let peer_username = match peer_register_response {
        InternalServiceResponse::PeerRegisterSuccess(PeerRegisterSuccess {
            cid: _,
            peer_cid: _,
            peer_username,
            request_id: _,
        }) => {
            println!("Accepted Registration Request from {peer_username:?}");
            peer_username
        }
        _ => {
            panic!("Unexpected Response to Peer Registration Attempt {peer_register_response:?}")
        }
    };

    // Receive Request to Connect
    let inbound_connection_request = service_connector.stream.next().await.unwrap();
    match inbound_connection_request {
        InternalServiceResponse::PeerConnectNotification(PeerConnectNotification {
            cid: _,
            peer_cid: _,
            session_security_settings: _,
            udp_mode: _,
            request_id: _,
        }) => {
            println!("Received Connection Request from {peer_username:?}")
        }
        _ => {
            panic!("Unexpected Response {inbound_connection_request:?}")
        }
    }

    // Accept Register Request from Peer One
    let peer_connect_request = InternalServiceRequest::PeerConnect {
        request_id: Uuid::new_v4(),
        cid,
        peer_cid,
        udp_mode: Default::default(),
        session_security_settings: Default::default(),
    };
    service_connector
        .sink
        .send(peer_connect_request)
        .await
        .unwrap();
    let peer_connect_response = service_connector.stream.next().await.unwrap();
    match peer_connect_response {
        InternalServiceResponse::PeerConnectSuccess(PeerConnectSuccess {
            cid: _,
            request_id: _,
        }) => {
            println!("Accepted Connection with {peer_username:?}")
        }
        _ => {
            panic!("Unexpected Response to Peer Connection Attempt {peer_connect_response:?}")
        }
    }

    // Give some time to ensure connection is established
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let inbound_message = service_connector.stream.next().await.unwrap();
    match inbound_message {
        InternalServiceResponse::MessageNotification(MessageNotification {
            message,
            cid: _,
            peer_cid: _,
            request_id: _,
        }) => {
            println!("Received Message: {message:?}");
        }
        _ => {
            panic!("Unexpected Response {inbound_message:?}")
        }
    }
}
