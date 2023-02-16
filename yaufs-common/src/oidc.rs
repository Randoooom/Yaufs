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

use crate::error::Result;
use crate::map_internal_error;
use async_once::AsyncOnce;
use cached::proc_macro::once;
use lazy_static::lazy_static;
use zitadel::credentials::{Application, AuthenticationOptions, ServiceAccount};
use zitadel::oidc::discovery::ZitadelProviderMetadata;
use zitadel::oidc::introspection::{AuthorityAuthentication, ZitadelIntrospectionResponse};

const ISSUER: &str = "OIDC_ISSUER";
const SERVICE_ACCOUNT: &str = "OIDC_SERVICE_ACCOUNT_KEY_PATH";
const APPLICATION: &str = "OIDC_APPLICATION_KEY_PATH";

#[derive(Clone, Debug)]
pub struct OIDCClient {
    authority_authentication: AuthorityAuthentication,
    service_account: ServiceAccount,
    metadata: ZitadelProviderMetadata,
    authentication_options: AuthenticationOptions,
}

#[once(time = 1800)]
pub async fn obtain_access_token(
    service_account: &ServiceAccount,
    issuer: &str,
    authentication_options: &AuthenticationOptions,
) -> String {
    service_account
        .authenticate_with_options(issuer, authentication_options)
        .await
        .unwrap()
}

impl OIDCClient {
    /// Create a new instance based on the set env variables. This will panic if they're set in an
    /// incompatible matter since the security of all applications rely on it.
    pub async fn new_from_env(roles: Vec<String>) -> Result<Self> {
        // access the process env vars
        let issuer = std::env::var(ISSUER).unwrap_or_else(|_| panic!("missing env var {ISSUER}"));
        let service_account_key_path = std::env::var(SERVICE_ACCOUNT)
            .unwrap_or_else(|_| panic!("missing env var {SERVICE_ACCOUNT}"));
        let application_key_path =
            std::env::var(APPLICATION).unwrap_or_else(|_| panic!("missing env var {APPLICATION}"));

        // read the key files
        let service_account = ServiceAccount::load_from_file(service_account_key_path.as_str())
            .unwrap_or_else(|error| {
                panic!("Error occured while loading service account key from file: {error}")
            });
        let application = Application::load_from_file(application_key_path.as_str())
            .unwrap_or_else(|error| {
                panic!("Error occured while loading application key from file: {error}")
            });

        // fetch the metadata
        let metadata = zitadel::oidc::discovery::discover(issuer.as_str())
            .await
            .unwrap_or_else(|error| {
                panic!("Error occured while discovering the oidc endpoints: {error}")
            });

        let authentication_options = AuthenticationOptions {
            api_access: false,
            scopes: Vec::new(),
            roles,
            project_audiences: Vec::new(),
        };

        Ok(Self {
            authority_authentication: AuthorityAuthentication::JWTProfile { application },
            service_account,
            metadata,
            authentication_options,
        })
    }

    /// Gain a new access token from the oidc provider. The access token will be cached
    /// for an half hour by the cached macro annotation on the sub function. This optimizes
    /// the time usage of all incoming requests overall.
    pub async fn obtain_access_token(&self) -> Result<String> {
        Ok(obtain_access_token(
            &self.service_account,
            self.metadata.issuer().as_str(),
            &self.authentication_options,
        )
        .await)
    }

    /// Introspect a given access token. This validates the integrity of an sent access token
    /// on the oidc provider. Caching is not supported due the ability of revocation.
    pub async fn introspect(&self, token: &str) -> Result<ZitadelIntrospectionResponse> {
        map_internal_error!(
            zitadel::oidc::introspection::introspect(
                self.metadata
                    .additional_metadata()
                    .introspection_endpoint
                    .as_ref()
                    .unwrap()
                    .as_str(),
                self.metadata.issuer().as_str(),
                &self.authority_authentication,
                token
            )
            .await,
            "Error occurred while calling introspection endpoint"
        )
    }
}

lazy_static! {
    pub static ref OIDC_CLIENT: AsyncOnce<OIDCClient> =
        AsyncOnce::new(async { OIDCClient::new_from_env(Vec::new()).await.unwrap() });
}
