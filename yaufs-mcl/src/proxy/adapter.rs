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

use crate::proxy::interceptor::PacketInterceptor;
use crate::proxy::PeerMap;
use kanal::{AsyncReceiver, AsyncSender};
use std::net::SocketAddr;
use tokio::io::BufReader;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use yaufs_common::craftio_rs::{
    CraftAsyncReader, CraftAsyncWriter, CraftConnection, CraftIo, CraftReader,
    CraftTokioConnection, CraftWriter,
};
use yaufs_common::net::packet::{Packet762, RawPacket762};

pub struct Adapter<
    W: CraftAsyncWriter + CraftIo + Send + 'static,
    R: CraftAsyncReader + CraftIo + Send + 'static,
> {
    pub client_address: SocketAddr,
    pub peers: PeerMap,
    pub writer: W,
    pub reader: R,
    client: bool,
}

pub type ServerAdapter =
    Adapter<CraftWriter<OwnedWriteHalf>, CraftReader<BufReader<OwnedReadHalf>>>;
pub type ClientAdapter = Adapter<CraftWriter<OwnedWriteHalf>, CraftReader<OwnedReadHalf>>;

impl TryFrom<(CraftTokioConnection, PeerMap, SocketAddr)>
    for Adapter<CraftWriter<OwnedWriteHalf>, CraftReader<BufReader<OwnedReadHalf>>>
{
    type Error = anyhow::Error;

    fn try_from(
        (stream, peers, client_address): (CraftTokioConnection, PeerMap, SocketAddr),
    ) -> Result<Self, Self::Error> {
        let (reader, writer) = stream.into_split();

        Ok(Self {
            client_address,
            peers,
            writer,
            reader,
            client: false,
        })
    }
}

impl
    TryFrom<(
        CraftConnection<OwnedReadHalf, OwnedWriteHalf>,
        PeerMap,
        SocketAddr,
    )> for Adapter<CraftWriter<OwnedWriteHalf>, CraftReader<OwnedReadHalf>>
{
    type Error = anyhow::Error;

    fn try_from(
        (connection, peers, address): (
            CraftConnection<OwnedReadHalf, OwnedWriteHalf>,
            PeerMap,
            SocketAddr,
        ),
    ) -> Result<Self, Self::Error> {
        let (reader, writer) = connection.into_split();

        Ok(Self {
            client: true,
            client_address: address,
            peers,
            writer,
            reader,
        })
    }
}

impl ServerAdapter {
    pub async fn run(
        mut self,
        receiver: AsyncReceiver<Packet762>,
        sender: AsyncSender<Packet762>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                message = receiver.recv() => {
                    match message {
                        Ok(packet) => {
                            self.on_send(packet).await?;
                        },
                        Err(_) => {
                            receiver.close();
                            break;
                        }
                    }
                },
                message = self.reader.read_packet_async::<RawPacket762>() => {
                    match message {
                        Ok(Some(packet)) => {
                            self.on_receive(packet, sender.clone()).await?;
                        },
                        Ok(None) => {
                            sender.close();
                            break;
                        },
                        Err(error) => {
                            error!("Error while receiving packet [{:?}]: {:?}", self.client, error);
                            sender.close();
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ClientAdapter {
    pub async fn run(
        mut self,
        receiver: AsyncReceiver<Packet762>,
        sender: AsyncSender<Packet762>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                message = receiver.recv() => {
                    match message {
                        Ok(packet) => {
                            self.on_send(packet).await?;
                        },
                        Err(_) => {
                            receiver.close();
                            break;
                        }
                    }
                },
                message = self.reader.read_packet_async::<RawPacket762>() => {
                    match message {
                        Ok(Some(packet)) => {
                            self.on_receive(packet, sender.clone()).await?;
                        },
                        Ok(None) => {
                            sender.close();
                            break;
                        },
                        Err(error) => {
                            error!("Error while receiving packet [{:?}]: {:?}", self.client, error);
                            sender.close();
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<
        W: CraftAsyncWriter + CraftIo + Send + 'static,
        R: CraftAsyncReader + CraftIo + Send + 'static,
    > Adapter<W, R>
{
    pub async fn send_packet(&mut self, packet: Packet762) -> anyhow::Result<()> {
        self.writer.write_packet_async(packet).await?;

        Ok(())
    }
}
