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

use crate::error::YaufsError;
use crate::oidc::OIDCClient;
use hyper::header::AUTHORIZATION;
use hyper::{Body, Request};
use std::error::Error;
use std::task::{Context, Poll};
use tonic::codegen::BoxFuture;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct AuthenticationLayer {
    client: OIDCClient,
}

impl From<OIDCClient> for AuthenticationLayer {
    fn from(client: OIDCClient) -> Self {
        Self { client }
    }
}

impl<S> Layer<S> for AuthenticationLayer {
    type Service = OIDCMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        OIDCMiddleware {
            inner: service,
            client: self.client.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OIDCMiddleware<S> {
    inner: S,
    client: OIDCClient,
}

impl<S> Service<Request<Body>> for OIDCMiddleware<S>
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
        let client = self.client.clone();
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            let span = tracing::info_span!("Authorizing request");
            let _ = span.enter();

            tracing::info!("{:?}", request.headers().get(AUTHORIZATION));
            // extract the Authorization header
            match request.headers().get(AUTHORIZATION) {
                Some(value) => {
                    let introspection_span = tracing::info_span!("Introspecting token");
                    let introspection_guard = introspection_span.enter();

                    let span = tracing::info_span!("Calling introspection endpoint");
                    let guard = span.enter();
                    // parse the token as str
                    let token = value.to_str().map_err(|_| YaufsError::Unauthorized)?;
                    // introspect the given token
                    client.introspect_token_valid(token).await?;
                    drop(guard);
                    drop(introspection_guard);

                    // call the next layer and return the response
                    let response = inner.call(request).await.map_err(Into::into)?;
                    Ok(response)
                }
                None => Err(YaufsError::Unauthorized)?,
            }
        })
    }
}
