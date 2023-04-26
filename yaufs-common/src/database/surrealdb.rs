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
use surrealdb::Surreal;
use version_compare::{Cmp, Version};

const SURREALDB_ENDPOINT: &str = "SURREALDB_ENDPOINT";
const SURREALDB_USERNAME: &str = "SURREALDB_USERNAME";
const SURREALDB_PASSWORD: &str = "SURREALDB_PASSWORD";

pub async fn connect(up: &'static str) -> Result<Surreal<Client>, Box<dyn std::error::Error>> {
    // establish the connection
    let client: Surreal<Client> = Surreal::new::<Ws>(
        std::env::var(SURREALDB_ENDPOINT)
            .unwrap_or_else(|_| panic!("Missing {SURREALDB_ENDPOINT} env variable")),
    )
    .await?;
    tracing::info!("Established connection to surrealdb");

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
    tracing::info!("Authenticated with surrealdb");

    // use namespace and database
    cfg_if::cfg_if! {
        if #[cfg(feature = "testing")] {
            let db = nanoid::nanoid!();
            tracing::info!("Using database {db}");

            client
                .use_ns("test")
                .use_db(db)
                .await?;
        } else {
            client
                .use_ns("production")
                .use_db("template-service")
                .await?;
        }
    }

    // execute the up queries
    client.query(up).await?;
    tracing::info!("Initiated tables");

    Ok(client)
}

pub async fn migrate(
    client: &Surreal<Client>,
    current_version: &'static str,
    migrations: Vec<(&'static str, &'static str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // initiate the migration table and fetch possibly already existing records
    let mut responses = client
        .query(
            "DEFINE TABLE migration SCHEMALESS;
            DEFINE FIELD version     on TABLE migration TYPE string ASSERT $value IS NOT NULL;
            DEFINE FIELD created_at  on TABLE migration TYPE datetime VALUE time::now();",
        )
        .query("SELECT version, created_at FROM migration ORDER BY created_at DESC LIMIT 1")
        .await?;
    // take the last as response, which contains the last migrated version
    let last = responses.take::<Option<String>>((1, "version"))?;

    if let Some(last) = last {
        // only proceed if the  last version is not equal to the current version
        if !last.as_str().eq(current_version) {
            // iterate through the given migrations
            for (version, migration) in migrations {
                if Version::from(last.as_str())
                    .unwrap()
                    .compare_to(Version::from(current_version).unwrap(), Cmp::Lt)
                {
                    tracing::info!("Executing surrealdb migration to {version}");
                    // execute the migration query and mark it as done
                    client
                        .query(migration)
                        .query("CREATE migration SET version = $version")
                        .bind(("version", version))
                        .await?;
                }
            }
        }
    } else {
        // insert the current version as the last version
        client
            .query("CREATE migration SET version = $version")
            .bind(("version", current_version))
            .await?;
    }

    Ok(())
}

#[macro_export]
macro_rules! sql_span {
    ($expr: expr) => {{
        let span = info_span!("Surrealdb Request");
        let _ = span.enter();
        $expr
    }};
    ($expr: expr, $title: expr) => {{
        let span = info_span!(concat!("Surrealdb Request: ", $title));
        let _ = span.enter();
        $expr
    }};
}
