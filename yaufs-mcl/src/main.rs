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

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate getset;
extern crate core;

mod proxy;

const ADDRESS: &str = "0.0.0.0:25565";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    yaufs_common::init_telemetry!();

    // start the proxy
    let proxy = proxy::ProxySocket::new();
    let proxy = tokio::spawn(async move { proxy.start().await });
    let _ = proxy.await;

    Ok(())
}
