use citadel_proto::prelude::NodeType;
use citadel_sdk::prefabs::server::client_connect_listener::ClientConnectListenerKernel;
use citadel_sdk::prelude::*;
use futures::StreamExt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use citadel_internal_service::*;
use citadel_internal_service::kernel::*;
use citadel_sdk::prefabs::ClientServerRemote;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    // Server
    let tcp_listener = std::net::TcpListener::bind("127.0.0.1:23458").unwrap();
    let bind_addr = tcp_listener.local_addr().unwrap();
    let kernel = Box::new(ClientConnectListenerKernel::new(
        move |connect_success, remote| async move {
            let client_cid = connect_success.cid;
            println!("Hello World! From Client {client_cid:?}");
            Ok(())
        }
    ));// as Box<dyn NetKernel>;
    let mut builder = NodeBuilder::default();
    let builder = builder
        .with_node_type(NodeType::Server(bind_addr))
        .with_insecure_skip_cert_verification()
        .with_underlying_protocol(
            ServerUnderlyingProtocol::from_tcp_listener(tcp_listener).unwrap(),
        );

    let server = builder.build(kernel).unwrap();
    tokio::task::spawn(server);

    // Internal Service
    let bind_address_internal_service: SocketAddr = "127.0.0.1:23457".parse().unwrap();
    let internal_service_kernel = CitadelWorkspaceService::new(bind_address_internal_service);
    let internal_service = NodeBuilder::default()
        .with_node_type(NodeType::Peer)
        .with_backend(BackendType::InMemory)
        .with_insecure_skip_cert_verification()
        .build(internal_service_kernel).unwrap();
    tokio::task::spawn(internal_service);

}