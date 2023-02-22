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

pub struct YaufsEvent;

impl YaufsEvent {
    /// Event issued on creation of a template by the template-service
    pub const TEMPLATE_CREATED: &'static str = "TEMPLATE_CREATED";
    /// Event issued on deletion of a template by the template-service
    pub const TEMPLATE_DELETED: &'static str = "TEMPLATE_DELETED";

    pub const INSTANCE_DEPLOYED: &'static str = "INSTANCE_DEPLOYED";
    pub const INSTANCE_STARTED: &'static str = "INSTANCE_STARTED";
    pub const INSTANCE_STOPPED: &'static str = "INSTANCE_STOPPED";
}

macro_rules! event {
    ($name:ident, $id: ident) => {
        event!(
            pub struct $name {
                $id: String,
            }
        );

        impl $name {
            pub fn new<S>(s: S) -> Self
            where
                S: Into<String>,
            {
                Self { $id: s.into() }
            }
        }
    };
    (
        $(
            pub struct $name:ident {
                $($field:ident: $ty:ty,)*
            }
        )*
    ) => {
        $(
            #[derive(Deserialize, Serialize, Debug, Clone)]
            pub struct $name {
                $(pub $field: $ty),*
            }

            impl Into<Vec<u8>> for $name {
                fn into(self) -> Vec<u8> {
                    serde_json::to_vec(&self).unwrap()
                }
            }
        )*
    };
}

event!(TemplateCreated, template_id);
event!(TemplateDeleted, template_id);
event!(InstanceStarted, id);

event!(
    pub struct InstanceDeployed {
        id: String,
        issuer: Option<String>,
    }

    pub struct InstanceStopped {
        id: String,
        issuer: Option<String>,
    }
);
