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

use crate::proxy::adapter::ClientAdapter;
use crate::proxy::interceptor::PacketInterceptor;
use crate::proxy::{ENCRYPTION_PRIVATE_KEY, ENCRYPTION_PUBLIC_KEY_BYTES};
use kanal::AsyncSender;
use rsa::Pkcs1v15Encrypt;
use yaufs_common::craftio_rs::CraftIo;
use yaufs_common::net::packet::{
    HandshakeNextState, LoginEncryptionRequestSpec, LoginStartSpec, LoginSuccessPropertiesSpec,
    LoginSuccessSpec, Packet762, StatusPongSpec, StatusResponseSpec,
};
use yaufs_common::protocol::State;
use yaufs_common::status::{StatusPlayersSpec, StatusSpec};
use yaufs_common::types::{BaseComponent, Chat, CountedArray, TextComponent};

#[async_trait]
impl PacketInterceptor for ClientAdapter {
    async fn on_receive(
        &mut self,
        packet: Packet762,
        sender: AsyncSender<Packet762>,
    ) -> anyhow::Result<()> {
        match &packet {
            Packet762::StatusPing(request) => {
                let packet = Packet762::StatusPong(StatusPongSpec {
                    payload: request.payload,
                });

                self.send_packet(packet).await?;
            }
            Packet762::StatusRequest(_) => {
                let packet = Packet762::StatusResponse(StatusResponseSpec {
                    response: StatusSpec {
                        version: None,
                        players: StatusPlayersSpec {
                            max: 0,
                            online: 1,
                            sample: vec![],
                        },
                        description: Chat::Text(TextComponent {
                            text: "Hey folks".to_owned(),
                            base: BaseComponent::default(),
                        }),
                        favicon: None,
                    },
                });

                self.send_packet(packet).await?;
            }
            Packet762::Handshake(handshake) => {
                let state = match handshake.next_state {
                    HandshakeNextState::Status => State::Status,
                    HandshakeNextState::Login => State::Login,
                };

                self.reader.set_state(state);
                self.writer.set_state(state);

                let mut peers = self.peers.lock().await;
                let connection = peers.get_mut(&self.client_address).unwrap();
                connection.set_state(state);

                match state {
                    State::Login => {
                        sender.send(packet).await?;
                    }
                    _ => {}
                }
            }
            Packet762::LoginStart(request) => {
                let mut peers = self.peers.lock().await;
                let connection = peers.get_mut(&self.client_address).unwrap();

                // request encryption
                let mut buffer = [0; 4];
                openssl::rand::rand_bytes(&mut buffer)?;
                let verify_token = CountedArray::from(Vec::from(buffer.as_slice()));
                connection.set_client_verify_token(Some(verify_token.clone()));
                connection.set_login(Some(request.clone()));
                drop(peers);

                let encryption_request =
                    Packet762::LoginEncryptionRequest(LoginEncryptionRequestSpec {
                        server_id: "".to_owned(),
                        verify_token,
                        public_key: CountedArray::from(ENCRYPTION_PUBLIC_KEY_BYTES.clone()),
                    });
                self.send_packet(encryption_request).await?;
            }
            Packet762::LoginEncryptionResponse(response) => {
                // TODO: validate token TODO: cleanup the whole code
                // let verify_token = ENCRYPTION_PRIVATE_KEY
                //     .decrypt(Pkcs1v15Encrypt, &response.verify_token)
                //     .unwrap();

                // decode the secret
                let secret = ENCRYPTION_PRIVATE_KEY
                    .decrypt(Pkcs1v15Encrypt, &response.shared_secret)
                    .unwrap();

                self.reader.enable_encryption(&secret, &secret)?;
                self.writer.enable_encryption(&secret, &secret)?;

                // authorize the login request
                let mut peers = self.peers.lock().await;
                let connection = peers.get_mut(&self.client_address).unwrap();
                let login_request = connection.login().as_ref().unwrap();

                // generate the hash
                let server_hash = mojang_api::server_hash(
                    "",
                    <[u8; 16]>::try_from(secret.as_slice())?,
                    &ENCRYPTION_PUBLIC_KEY_BYTES,
                );
                // verify the session
                let authentication_response = reqwest::get(
                    format!(
                        "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
                        login_request.name.as_str(),
                        server_hash.as_str())
                    )
                    .await?.json::<mojang_api::ServerAuthResponse>().await?;
                sender
                    .send(Packet762::LoginStart(LoginStartSpec {
                        name: login_request.name.to_string(),
                        has_uuid: login_request.has_uuid.clone(),
                        uuid: login_request.uuid.clone(),
                    }))
                    .await?;
                let uuid = login_request.uuid.clone();
                drop(peers);

                self.send_packet(Packet762::LoginSuccess(LoginSuccessSpec {
                    uuid,
                    username: authentication_response.name.clone(),
                    properties: CountedArray::from(
                        authentication_response
                            .properties
                            .into_iter()
                            .map(|properties| LoginSuccessPropertiesSpec {
                                name: properties.name,
                                value: properties.value,
                                signed: true,
                                signature: properties.signature,
                            })
                            .collect::<Vec<LoginSuccessPropertiesSpec>>(),
                    ),
                }))
                .await?;
                self.reader.set_state(State::Play);
                self.writer.set_state(State::Play);
            }
            _ => {
                sender.send(packet).await?;
            }
        };

        Ok(())
    }

    async fn on_send(&mut self, packet: Packet762) -> anyhow::Result<()> {
        match &packet {
            Packet762::LoginSetCompression(request) => {
                let threshold = Some(request.threshold.0);
                self.send_packet(packet).await?;

                self.writer.set_compression_threshold(threshold);
                self.reader.set_compression_threshold(threshold);
            }
            _ => {
                self.send_packet(packet).await?;
            }
        }

        Ok(())
    }
}
