use citadel_proto::prelude::{
    EncryptionAlgorithm, KemAlgorithm, SecrecyMode, SecureProtocolPacket,
    SessionSecuritySettingsBuilder, UdpMode,
};
use citadel_sdk::prefabs::client::single_connection::SingleClientServerConnectionKernel;
use citadel_sdk::prelude::NodeBuilder;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;
use citadel_internal_service::*;
use citadel_internal_service::kernel::*;
use citadel_internal_service_connector::util::*;
use citadel_internal_service_types::*;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    // Connect to Internal Service via TCP
    let bind_address_internal_service: SocketAddr = "127.0.0.1:23457".parse().unwrap();
    let tcp_conn = TcpStream::connect(bind_address_internal_service).await.unwrap();
    let (mut sink, mut stream) = wrap_tcp_conn(tcp_conn).split();

    // Receive Greeter Packet
    let first_packet = stream.next().await.unwrap().unwrap();
    let greeter_packet: InternalServiceResponse = bincode2::deserialize(&first_packet).unwrap();
    if let InternalServiceResponse::ServiceConnectionAccepted(ServiceConnectionAccepted) = greeter_packet
    {
        println!("Client Successfully Connected To Internal Service");
    } else {
        panic!("Error occurred while trying to connect to Internal Service");
    }

    // Register To and Connect To Server
    let session_security_settings = SessionSecuritySettingsBuilder::default()
        .build()
        .unwrap();
    let register_request = InternalServiceRequest::Register {
        request_id: Uuid::new_v4(),
        server_addr: "127.0.0.1:23458".parse().unwrap(),
        full_name: "Client One".parse().unwrap(),
        username: "ClientOne".parse().unwrap(),
        proposed_password: "secret".into(),
        session_security_settings,
        connect_after_register: true,
    };
    sink.send(register_request.into()).await.unwrap();
    let serialized_response = stream.next().await.unwrap().unwrap();
    let register_response = deserialize(&serialized_response).unwrap();
    match register_response {
        InternalServiceResponse::RegisterSuccess( RegisterSuccess { request_id: _ } ) => {
            println!("Client Successfully Registered and Connected to Server")
        },

        InternalServiceResponse::RegisterFailure( RegisterFailure { message, request_id: _ } ) => {
            println!("Client Failed to Register/Connect to Server: {message:?}")
        },

        _ => {
            panic!("Unhandled Response While Trying to Register/Connect to Server")
        },
    }

}