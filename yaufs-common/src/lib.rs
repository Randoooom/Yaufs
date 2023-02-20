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

#[cfg(feature = "fluvio")]
pub extern crate fluvio;
#[cfg(feature = "skytable")]
pub extern crate skytable;
#[cfg(feature = "surrealdb")]
pub extern crate surrealdb;
pub extern crate yaufs_proto;

pub mod database;
pub mod error;
#[cfg(feature = "fluvio")]
pub mod fluvio_util;
pub mod oidc;
pub mod telemetry;
pub mod tonic;
pub mod tower;

mod util;
