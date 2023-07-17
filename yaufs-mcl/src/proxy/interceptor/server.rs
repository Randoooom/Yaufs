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

use crate::proxy::adapter::ServerAdapter;
use crate::proxy::interceptor::PacketInterceptor;
use kanal::AsyncSender;
use yaufs_common::craftio_rs::CraftIo;
use yaufs_common::net::packet::Packet762;
use yaufs_common::protocol::State;

#[async_trait]
impl PacketInterceptor for ServerAdapter {
    async fn on_receive(
        &mut self,
        packet: Packet762,
        sender: AsyncSender<Packet762>,
    ) -> anyhow::Result<()> {
        match &packet {
            Packet762::LoginSetCompression(compression) => {
                self.reader
                    .set_compression_threshold(Some(compression.threshold.0));
                self.writer
                    .set_compression_threshold(Some(compression.threshold.0));
            }
            Packet762::LoginSuccess(_) => {
                self.reader.set_state(State::Play);
                self.writer.set_state(State::Play);
            }
            _ => {
                sender.send(packet).await?;
            }
        }

        Ok(())
    }

    async fn on_send(&mut self, packet: Packet762) -> anyhow::Result<()> {
        match &packet {
            Packet762::LoginStart(_) => {
                self.reader.set_state(State::Login);
                self.writer.set_state(State::Login);
            }
            _ => {}
        }

        self.send_packet(packet).await?;
        Ok(())
    }
}
