use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerConfig};

fn main() {
    let mut server = RenetServer::new(ConnectionConfig::default());

    // The transport layer. Later, I can convert this to use renet_steam.
    const SERVER_ADRESS: &str = "127.0.0.1:3145";
    let socket_adress = SERVER_ADRESS.parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(socket_adress).expect("could not bind to this adress");
    let mut transport = NetcodeServerTransport::new(
        ServerConfig {
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            max_clients: 6,
            protocol_id: PROTOCOL_ID,
            public_addresses: vec![socket_adress],
            authentication: renet_netcode::ServerAuthentication::Unsecure,
        },
        socket,
    )
    .unwrap();

    loop {
        // The sever will run this code thirty times per second (30 Hz)
        let delta_time = Duration::from_millis(32);

        server.update(delta_time);
        transport.update(delta_time, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("Client {client_id} connected");
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("Client {client_id} disconnected: {reason}");
                }
            }
        }

        for client in server.clients_id() {
            while let Some(message) =
                server.receive_message(client, DefaultChannel::ReliableOrdered)
            {
                println!("Received a message from a client.")
            }
        }

        std::thread::sleep(delta_time);
    }
}

pub const PROTOCOL_ID: u64 = 1892;
pub const MAX_PLAYERS: u64 = 6;
