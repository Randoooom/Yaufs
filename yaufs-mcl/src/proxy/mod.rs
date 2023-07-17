/*
 *    Copyright  2023.  Fritz Ochsmann
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use crate::proxy::adapter::Adapter;
use crate::proxy::connection::ProxyConnection;
use crate::ADDRESS;
use rsa::pkcs8::EncodePublicKey;
use rsa::rand_core::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use yaufs_common::craftio_rs::{CraftConnection, CraftTokioConnection};
use yaufs_common::mcproto_rs::protocol::PacketDirection;
use yaufs_common::net::packet::Packet762;
use yaufs_common::protocol::State;

mod adapter;
mod connection;
mod interceptor;

// TODO: may consider to save the information in skytable in order to be able to run multiple instances
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, ProxyConnection>>>;

// this generates a new rsa keypair on proxy startup as specified in https://wiki.vg/Protocol_Encryption
lazy_static::lazy_static! {
    pub static ref ENCRYPTION_PRIVATE_KEY: RsaPrivateKey = {
        RsaPrivateKey::new(&mut OsRng, 1024).unwrap()
    };
    pub static ref ENCRYPTION_PUBLIC_KEY: RsaPublicKey = {
        RsaPublicKey::from(ENCRYPTION_PRIVATE_KEY.deref())
    };

    pub static ref ENCRYPTION_PUBLIC_KEY_BYTES: Vec<u8> = {
        ENCRYPTION_PUBLIC_KEY.deref().to_public_key_der().unwrap().to_vec()
    };
}

macro_rules! connector {
    ($adapter:expr, $receiver:expr, $sender:expr) => {
        tokio::spawn(async move { $adapter.run($receiver, $sender).await })
    };
}

#[derive(Clone)]
pub struct ProxySocket {
    peers: PeerMap,
}

impl ProxySocket {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(self) {
        let socket = TcpListener::bind(ADDRESS)
            .await
            .expect("Error while binding to address");
        info!("Listening for incoming connections on {}", ADDRESS);

        while let Ok((stream, address)) = socket.accept().await {
            let context = self.clone();
            tokio::spawn(
                async move { ProxySocket::handle_connection(context, stream, address).await },
            );
        }
    }

    async fn handle_connection(self, stream: TcpStream, address: SocketAddr) -> anyhow::Result<()> {
        debug!("Incoming connection from {}", address);
        let (read, write) = stream.into_split();
        let craft_stream = CraftConnection::from_async_with_state(
            (read, write),
            PacketDirection::ServerBound,
            State::Handshaking,
        );
        self.peers
            .lock()
            .await
            .insert(address.clone(), ProxyConnection::default());

        let (client_write_sender, client_write_receiver) = kanal::unbounded_async::<Packet762>();
        let (server_write_sender, server_write_receiver) = kanal::unbounded_async::<Packet762>();

        let client_adapter =
            Adapter::try_from((craft_stream, self.peers.clone(), address.clone())).unwrap();
        let server_address = SocketAddr::new("127.0.0.1".parse()?, 25566);
        let server_listener = CraftTokioConnection::connect_server_tokio(server_address).await?;
        let server_adapter =
            Adapter::try_from((server_listener, self.peers.clone(), address.clone())).unwrap();

        // start the process
        let server_connector =
            connector!(server_adapter, server_write_receiver, client_write_sender);
        let client_connector =
            connector!(client_adapter, client_write_receiver, server_write_sender);

        client_connector.await.unwrap().unwrap();
        server_connector.await.unwrap().unwrap();

        debug!("Client disconnected from {}", address);
        // remove the disconnected client from the peer map
        self.peers.lock().await.remove(&address);

        Ok(())
    }
}
