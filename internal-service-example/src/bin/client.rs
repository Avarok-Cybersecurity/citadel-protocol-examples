use citadel_proto::prelude::{
    EncryptionAlgorithm, KemAlgorithm, SecrecyMode, SecureProtocolPacket,
    SessionSecuritySettingsBuilder, UdpMode,
};
use citadel_sdk::prefabs::client::single_connection::SingleClientServerConnectionKernel;
use citadel_sdk::prelude::NodeBuilder;
use futures::StreamExt;
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;
use citadel_internal_service::*;
use citadel_internal_service::kernel::*;

#[tokio::main]
async fn main() {
    // let (server, server_bind_address) = server_info_skip_cert_verification();
    // tokio::task::spawn(server);

}