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

use kanal::AsyncSender;
use yaufs_common::net::packet::Packet762;

mod client;
mod server;

#[async_trait]
pub trait PacketInterceptor {
    async fn on_receive(
        &mut self,
        packet: Packet762,
        sender: AsyncSender<Packet762>,
    ) -> anyhow::Result<()>;

    async fn on_send(&mut self, mut packet: Packet762) -> anyhow::Result<()>;
}
