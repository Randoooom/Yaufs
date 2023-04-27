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
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "surrealdb")]
use surrealdb::opt::{IntoResource, Resource};
#[cfg(feature = "surrealdb")]
use surrealdb::sql::Thing;

#[derive(Debug, Clone, PartialEq)]
pub struct Id {
    pub table: String,
    pub id: String,
}

impl TryFrom<(&str, &str)> for Id {
    type Error = YaufsError;

    fn try_from((force, id): (&str, &str)) -> Result<Self, Self::Error> {
        let mut split = id.split(':');
        let table = split.next().ok_or(YaufsError::NotFound("invalid id"))?;
        // for security reasons we can't allow every table
        if !table.eq(force) {
            return Err(YaufsError::Unauthorized);
        }

        let id = split.next().ok_or(YaufsError::NotFound("invalid id"))?;

        Ok(Self {
            table: table.to_string(),
            id: id.to_string(),
        })
    }
}

impl Id {
    pub fn new((table, id): (&str, &str)) -> Self {
        Self {
            table: table.to_string(),
            id: id.to_string(),
        }
    }

    pub fn new_random(table: &str) -> Self {
        Self {
            table: table.to_string(),
            id: nanoid!(),
        }
    }

    #[cfg(feature = "surrealdb")]
    pub fn to_thing(&self) -> Thing {
        Thing::from((self.table.as_str(), self.id.as_str()))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_value = serde_json::value::Value::deserialize(deserializer).unwrap();

        if let Some(string) = raw_value.as_str() {
            let mut split = string.split(':');
            let table = split
                .next()
                .ok_or(serde::de::Error::custom("Invalid id format"))?
                .to_string();
            let id = split
                .next()
                .ok_or(serde::de::Error::custom("Invalid id format"))?
                .to_string();

            return Ok(Self { table, id });
        }

        #[cfg(feature = "surrealdb")]
        if raw_value.is_object() {
            // deserialize it as `Thing`
            // TODO: map err
            let thing = serde_json::from_value::<Thing>(raw_value).unwrap();
            return Ok(Self {
                table: thing.tb,
                id: thing.id.to_string(),
            });
        }

        Err(serde::de::Error::custom("Invalid datatype"))
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        format!("{}:{}", &self.table, &self.id)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[cfg(feature = "surrealdb")]
impl<R> IntoResource<Option<R>> for &Id {
    fn into_resource(self) -> surrealdb::Result<Resource> {
        Ok(Resource::RecordId(self.to_thing()))
    }
}
