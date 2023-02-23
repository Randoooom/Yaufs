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

use generic_array::typenum::U40;
use generic_array::GenericArray;
use skytable::actions::AsyncActions;
use skytable::pool::AsyncPool;

const SKYTABLE_HOST: &str = "SKYTABLE_HOST";
const SKYTABLE_PORT: &str = "SKYTABLE_PORT";
const SKYTABLE_ORIGIN_KEY: &str = "SKYTABLE_ORIGIN_KEY";

pub async fn connect() -> AsyncPool {
    // read the configuration from the process environment
    let host = std::env::var(SKYTABLE_HOST)
        .unwrap_or_else(|_| panic!("Missing {SKYTABLE_HOST} env variable"));
    let port: u16 = std::env::var(SKYTABLE_PORT)
        .unwrap_or_else(|_| panic!("Missing {SKYTABLE_PORT} env variable"))
        .as_str()
        .parse()
        .unwrap_or_else(|_| panic!("Invalid value for {SKYTABLE_PORT}: expected u16"));
    let origin_key = std::env::var(SKYTABLE_ORIGIN_KEY)
        .unwrap_or_else(|_| panic!("Missing {SKYTABLE_ORIGIN_KEY} env variable"));
    let origin_key: GenericArray<u8, U40> = GenericArray::clone_from_slice(origin_key.as_bytes());

    // start a new connection to the skytable kv server
    tracing::info!("Connecting to skytable on {host}:{port}");
    let pool = skytable::pool::get_async(host, port, 10).await.unwrap();
    tracing::info!("Connected to skytable");
    let mut connection = pool.get().await.unwrap();

    // try to claim the root account
    let token = match connection.auth_claim(origin_key.into()).await {
        Ok(token) => token,
        Err(_) => {
            // the root was already claimed so we gonna reset it
            connection
                .auth_restore(origin_key.into(), "root")
                .await
                .unwrap()
        }
    };
    // login into the root account
    connection.auth_login("root", token.as_str()).await.unwrap();
    drop(connection);

    Ok(pool)
}

#[macro_export]
macro_rules! kv_span {
    ($expr: expr) => {{
        let span = info_span!("Skytable Request");
        let _ = span.enter();
        $expr
    }};
    ($expr: expr, $title: expr) => {{
        let span = info_span!(concat!("Skytable Request: ", $title));
        let _ = span.enter();
        $expr
    }};
}
