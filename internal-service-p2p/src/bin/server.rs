use citadel_proto::prelude::NodeType;
use citadel_sdk::prefabs::server::client_connect_listener::ClientConnectListenerKernel;
use citadel_sdk::prelude::*;
use structopt::{lazy_static, StructOpt};
use citadel_logging::{info, error};

#[tokio::main]
async fn main() {
    citadel_logging::setup_log();
    // Server
    let tcp_listener = std::net::TcpListener::bind("127.0.0.1:23458").unwrap();
    let bind_addr = tcp_listener.local_addr().unwrap();
    let kernel = Box::new(ClientConnectListenerKernel::new(
        move |connect_success, _remote| async move {
            let client_cid = connect_success.cid;
            println!("Hello World! From Client {client_cid:?}");
            Ok(())
        },
    ));
    let mut builder = NodeBuilder::default();
    let builder = builder
        .with_node_type(NodeType::Server(bind_addr))
        .with_insecure_skip_cert_verification()
        .with_underlying_protocol(
            ServerUnderlyingProtocol::from_tcp_listener(tcp_listener).unwrap(),
        );

    let server = builder.build(kernel).unwrap();
    let _ = server.await;
}
