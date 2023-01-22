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

use async_once::AsyncOnce;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use serde::{Deserialize, Serialize};

const ISSUER_URL: &str = "OIDC_ISSUER_URL";
const CLIENT_ID: &str = "OIDC_CLIENT_ID";
const CLIENT_SECRET: &str = "OIDC_CLIENT_SECRET";

lazy_static::lazy_static! {
    pub static ref OPENID_CLIENT: AsyncOnce<CoreClient> = AsyncOnce::new(async {
        let issuer = IssuerUrl::new(std::env::var(ISSUER_URL)
            .unwrap_or_else(|_| panic!("Missing {ISSUER_URL} env variable")))
            .unwrap();
        let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client).await.unwrap();

        let client_id = ClientId::new(std::env::var(CLIENT_ID)
            .unwrap_or_else(|_| panic!("Missing {CLIENT_ID} env variable")));
        let client_secret = ClientSecret::new(std::env::var(CLIENT_SECRET)
            .unwrap_or_else(|_| panic!("Missing {CLIENT_SECRET} env variable")));

        CoreClient::from_provider_metadata(metadata, client_id, Some(client_secret))
    });
}

pub struct AuthorizationScope;

/// All valid openid-connect scopes for the specified vault provider in the k8 cluster.
impl AuthorizationScope {
    pub const OPEN_ID: &'static str = "openid";
    pub const USER: &'static str = "user";
    pub const GROUPS: &'static str = "groups";
}

pub struct AuthorizationGroup;

/// These is a list of all identity groups which are associated with the local vault
/// provider in the k8 cluster.
impl AuthorizationGroup {
    pub const TEMPLATING: &'static str = "templating";
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AdditionalClaims {
    pub groups: Vec<String>,
}

impl openidconnect::AdditionalClaims for AdditionalClaims {}
