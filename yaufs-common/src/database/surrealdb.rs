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

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
// use surrealdb::{sql, Connection, Surreal};
use surrealdb::Surreal;

const SURREALDB_ENDPOINT: &str = "SURREALDB_ENDPOINT";
const SURREALDB_USERNAME: &str = "SURREALDB_USERNAME";
const SURREALDB_PASSWORD: &str = "SURREALDB_PASSWORD";

pub async fn connect(
    client: &'static Surreal<Client>,
    up: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // establish the connection
    client
        .connect::<Ws>(
            std::env::var(SURREALDB_ENDPOINT)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_ENDPOINT} env variable")),
        )
        .await?;

    // authenticate
    client
        .signin(Root {
            username: std::env::var(SURREALDB_USERNAME)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_USERNAME} env variable"))
                .as_str(),
            password: std::env::var(SURREALDB_PASSWORD)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_PASSWORD} env variable"))
                .as_str(),
        })
        .await?;

    // execute the up queries
    client.query(up).await?;

    Ok(())
}

// pub async fn migrate(
//     client: &'static Surreal<Client>,
//     init: &str,
//     migrations: Vec<(&str, &str)>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // initiate the migration table and fetch possibly already existing records
//     let mut responses = client
//         .query(sql!(include_str!("surrealql/migration.surrealql")))
//         .query("SELECT version as result FROM migration ORDER BY created_at DESC LIMIT 1")
//         .await?;
//     // take the last as response, which contains the last migrated version
//     let last = responses.take::<String>(1)?;
//
//     Ok(())
// }