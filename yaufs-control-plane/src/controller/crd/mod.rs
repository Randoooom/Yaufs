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

use crate::controller::ControlPlaneError;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Resource};
use serde::de::DeserializeOwned;

pub mod instance;
pub mod template;

pub enum CRDAction {
    Create,
    Delete,
    Update,
}

pub trait ActionDeterminable {
    fn determine_action(&self) -> CRDAction;
}

impl<T> ActionDeterminable for T
where
    T: Resource,
{
    fn determine_action(&self) -> CRDAction {
        if self.meta().deletion_timestamp.is_some() {
            CRDAction::Delete
        } else if self
            .meta()
            .finalizers
            .as_ref()
            .map_or(true, |finalizers| finalizers.is_empty())
        {
            CRDAction::Create
        } else {
            CRDAction::Update
        }
    }
}

/// Applies the finalizier to the specified crd
#[tracing::instrument(skip(client))]
pub async fn apply_finalizer<T>(
    name: &str,
    namespace: &str,
    client: Client,
) -> Result<(), ControlPlaneError>
where
    T: Resource<Scope = NamespaceResourceScope>,
    T::DynamicType: Default,
    T: DeserializeOwned,
    T: Clone,
    T: std::fmt::Debug,
{
    let api = Api::<T>::namespaced(client, namespace);
    let patch = serde_json::json!({
        "metadata": {
            "finalizers": ["yaufs.io/finalizer"]
        }
    });
    let patch = Patch::Merge(&patch);
    // patch the crd
    api.patch(name, &PatchParams::default(), &patch).await?;

    Ok(())
}

/// Removes the finalizier to the specified crd
#[tracing::instrument(skip(client))]
pub async fn remove_finalizer<T>(
    name: &str,
    namespace: &str,
    client: Client,
) -> Result<(), ControlPlaneError>
where
    T: Resource<Scope = NamespaceResourceScope>,
    T::DynamicType: Default,
    T: DeserializeOwned,
    T: Clone,
    T: std::fmt::Debug,
{
    let api = Api::<T>::namespaced(client, namespace);
    let patch = &serde_json::json!({
        "metadata": {
            "finalizers": null
        }
    });
    let patch = Patch::Merge(&patch);
    // patch the crd
    api.patch(name, &PatchParams::default(), &patch).await?;

    Ok(())
}
