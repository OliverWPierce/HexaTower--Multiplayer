use std::net::{SocketAddr, UdpSocket};

use renet::{ConnectionConfig, RenetServer};
use renet_netcode::{NetcodeServerTransport, ServerConfig};

fn main() {
    let mut server = RenetServer::new(ConnectionConfig::default());

    // The transport layer. Later, I can convert this to use renet_steam.
    const SERVER_ADRESS: &str = "127.0.0.1:3145";
    let socket_adress = SERVER_ADRESS.parse::<SocketAddr>().unwrap();

    let socket = UdpSocket::bind(socket_adress).expect("could not bind to this adress");
    let mut transport = NetcodeServerTransport::new();
}

pub const PROTOCOL_ID: u64 = 0892;
