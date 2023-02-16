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
use crate::oidc::OIDC_CLIENT;
use hyper::header::AUTHORIZATION;
use hyper::{Body, Request};
use openidconnect::{Scope, TokenIntrospectionResponse};
use std::error::Error;
use std::task::{Context, Poll};
use tonic::codegen::BoxFuture;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct AuthenticationLayer {
    roles: Vec<&'static str>,
}

impl From<Vec<&'static str>> for AuthenticationLayer {
    fn from(roles: Vec<&'static str>) -> Self {
        Self { roles }
    }
}

impl<S> Layer<S> for AuthenticationLayer {
    type Service = AuthenticationMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthenticationMiddleware {
            inner: service,
            roles: self
                .roles
                .clone()
                .into_iter()
                .map(|role| Scope::new(format!("urn:zitadel:iam:org:project:role:{}", { role })))
                .collect::<Vec<Scope>>(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<S> {
    inner: S,
    roles: Vec<Scope>,
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
        let roles = self.roles.clone();
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            // extract the Authorization header
            match request.headers().get(AUTHORIZATION) {
                Some(value) => {
                    // parse the token as str
                    let token = value.to_str().map_err(|_| YaufsError::Unauthorized)?;
                    // introspect the given token
                    let response = OIDC_CLIENT.get().await.introspect(token).await?;

                    // only allow further processing of the incoming request, if the given token
                    // is still in an active state and the token has the scopes for the required roles
                    let has_scopes = if let Some(scopes) = response.scopes() {
                        roles.iter().all(|role| scopes.contains(&role))
                    } else {
                        // we can omit false here as the scope 'openid' has to be everywhere
                        false
                    };
                    if !response.active() || !has_scopes {
                        return Err(YaufsError::Unauthorized)?;
                    };

                    // call the next layer and return the response
                    let response = inner.call(request).await.map_err(Into::into)?;
                    Ok(response)
                }
                None => Err(YaufsError::Unauthorized)?,
            }
        })
    }
}
