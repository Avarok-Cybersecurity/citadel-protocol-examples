use citadel_proto::prelude::NodeType;
use citadel_sdk::prefabs::server::client_connect_listener::ClientConnectListenerKernel;
use citadel_sdk::prelude::*;
use futures::StreamExt;
use std::net::SocketAddr;
use std::str::FromStr;
use citadel_internal_service::*;
use citadel_internal_service::kernel::*;

#[tokio::main]
async fn main() {
    let bind_address_internal_service: SocketAddr = "127.0.0.1:23456".parse().unwrap();
    let internal_service_kernel = CitadelWorkspaceService::new(bind_address_internal_service);
    let internal_service = NodeBuilder::default()
        .with_node_type(NodeType::Peer)
        .with_backend(BackendType::InMemory)
        .with_insecure_skip_cert_verification()
        .build(internal_service_kernel)?;

    match internal_service.await {
        Ok(_) => { std::println!("Success") }
        Err(err) => { std::println!("Error: {:?}", err) }
    }
}