use crate::SURREALDB;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql;

const SURREALDB_ENDPOINT: &str = "SURREALDB_ENDPOINT";
const SURREALDB_USERNAME: &str = "SURREALDB_USERNAME";
const SURREALDB_PASSWORD: &str = "SURREALDB_PASSWORD";

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    // establish the connection
    SURREALDB
        .connect::<Ws>(
            std::env::var(SURREALDB_ENDPOINT)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_ENDPOINT} env variable")),
        )
        .await?;

    // authenticate
    SURREALDB
        .signin(Root {
            username: std::env::var(SURREALDB_USERNAME)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_USERNAME} env variable"))
                .as_str(),
            password: std::env::var(SURREALDB_PASSWORD)
                .unwrap_or_else(|_| panic!("Missing {SURREALDB_PASSWORD} env variable"))
                .as_str(),
        })
        .await?;

    Ok(())
}

async fn migrate() -> Result<(), Box<dyn std::error::Error>> {
    // initiate the migration table and fetch possibly already existing records
    SURREALDB
        .query(sql!(include_str!("surrealql/migration.surrealql")))
        .query("")
        .await?;

    Ok(())
}
