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
    let bind_address_internal_service: SocketAddr = "127.0.0.1:23456".parse().unwrap();
    let internal_service_kernel = CitadelWorkspaceService::new(bind_address_internal_service);
    let internal_service = NodeBuilder::default()
        .with_node_type(NodeType::Peer)
        .with_backend(BackendType::InMemory)
        .with_insecure_skip_cert_verification()
        .build(internal_service_kernel)
        .unwrap();
    tokio::task::spawn(internal_service);

    // Connect to Internal Service via TCP
    let mut service_connector = InternalServiceConnector::connect("127.0.0.1:23456")
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
    let cid = match register_response {
        InternalServiceResponse::ConnectSuccess(
            citadel_internal_service_types::ConnectSuccess { cid, request_id: _ },
        ) => {
            println!("Client Successfully Connected to Server");
            cid
        }

        // Will panic if Client is already registered
        _ => {
            panic!("Unhandled Response While Trying to Register/Connect to Server: {register_response:?}")
        }
    };

    // Give time to ensure Peer Two is connected
    tokio::time::sleep(Duration::from_millis(5000)).await;

    // Get Peer CID from list of all Peers on Server
    let get_peers_request = InternalServiceRequest::ListAllPeers {
        request_id: Uuid::new_v4(),
        cid,
    };
    service_connector
        .sink
        .send(get_peers_request)
        .await
        .unwrap();
    let get_peers_response = service_connector.stream.next().await.unwrap();
    let peer_cid = match get_peers_response {
        InternalServiceResponse::ListAllPeers(ListAllPeers {
            cid: _,
            online_status,
            request_id: _,
        }) => *online_status.keys().next().unwrap(),
        _ => {
            panic!("Peer List Retrieval Failure")
        }
    };

    // Request to register with Peer Two
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
            println!("Requested to Register with {peer_username:?}");
            peer_username
        }
        _ => {
            panic!("Unexpected Response to Peer Registration Attempt {peer_register_response:?}")
        }
    };

    // Request to Connect to Peer Two
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
            println!("Requested to Connect to {peer_username:?}")
        }
        _ => {
            panic!("Unexpected Response to Peer Connect Attempt {peer_connect_response:?}")
        }
    }

    let peer_message_request = InternalServiceRequest::Message {
        request_id: Uuid::new_v4(),
        message: "Hello Peer Two! This is Peer One!".into(),
        cid,
        peer_cid: Some(peer_cid),
        security_level: Default::default(),
    };
    service_connector
        .sink
        .send(peer_message_request)
        .await
        .unwrap();
    let peer_message_response = service_connector.stream.next().await.unwrap();
    match peer_message_response {
        InternalServiceResponse::MessageSent(MessageSent {
            cid: _,
            peer_cid: _,
            request_id: _,
        }) => {
            println!("Successfully Sent Message to Peer Two")
        }
        _ => {
            panic!("Unexpected Response Following Peer Message Attempt {peer_message_response:?}")
        }
    }

    tokio::time::sleep(Duration::from_millis(2000)).await;
}
