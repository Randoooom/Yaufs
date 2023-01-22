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

use crate::auth::{AdditionalClaims, OPENID_CLIENT};
use crate::error::YaufsError;
use hyper::header::AUTHORIZATION;
use hyper::{Body, Request};
use openidconnect::core::CoreGenderClaim;
use openidconnect::reqwest::async_http_client;
use openidconnect::{AccessToken, TokenIntrospectionResponse, UserInfoClaims};
use std::error::Error;
use std::task::{Context, Poll};
use tonic::codegen::BoxFuture;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct AuthenticationLayer {
    group: Option<&'static str>,
}

impl From<Option<&'static str>> for AuthenticationLayer {
    fn from(group: Option<&'static str>) -> Self {
        tokio::spawn(async {
            OPENID_CLIENT.get().await;
        });

        Self { group }
    }
}

impl<S> Layer<S> for AuthenticationLayer {
    type Service = AuthenticationMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthenticationMiddleware {
            inner: service,
            group: self.group,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<S> {
    inner: S,
    group: Option<&'static str>,
}

impl<S> Service<Request<Body>> for AuthenticationMiddleware<S>
where
    S: Service<Request<Body>> + Send + Clone + 'static,
    S::Future: 'static + Send,
    S::Error: Into<Box<dyn Error + Send + Sync>> + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context).map_err(Into::into)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let group = self.group;
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            // extract the Authorization header
            match request.headers().get(AUTHORIZATION) {
                Some(value) => {
                    // verify the accessToken
                    let access_token = AccessToken::new(
                        value
                            .to_str()
                            .map_err(|_| YaufsError::Unauthorized)?
                            .to_string(),
                    );
                    let response = OPENID_CLIENT
                        .get()
                        .await
                        .introspect(&access_token)?
                        .request_async(async_http_client)
                        .await?;
                    // if the token is not valid anymore reject the request
                    if !response.active() {
                        return Err(YaufsError::Unauthorized)?;
                    }

                    // if the middleware has to verify the ownership of a role fetch the userinfo
                    if let Some(group) = group {
                        // fetch the userinfo
                        let claims: UserInfoClaims<AdditionalClaims, CoreGenderClaim> =
                            OPENID_CLIENT
                                .get()
                                .await
                                .user_info(access_token, None)?
                                .request_async::<_, _, _, CoreGenderClaim, _>(async_http_client)
                                .await?;

                        // search for the required group
                        if None
                            == claims
                                .additional_claims()
                                .groups
                                .iter()
                                .find(|current| current.as_str().eq(group))
                        {
                            return Err(YaufsError::Unauthorized)?;
                        }
                    }

                    // call the next layer and return the response
                    let response = inner.call(request).await.map_err(Into::into)?;
                    Ok(response)
                }
                None => Err(YaufsError::Unauthorized)?,
            }
        })
    }
}
