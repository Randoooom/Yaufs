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

use crate::prelude::*;
use template_service_v1_server::{TemplateServiceV1, TemplateServiceV1Server};

pub struct TemplateServiceV1Context;

pub type Server = TemplateServiceV1Server<TemplateServiceV1Context>;

pub fn new() -> Server {
    TemplateServiceV1Server::new(TemplateServiceV1Context)
}

#[async_trait]
impl TemplateServiceV1 for TemplateServiceV1Context {
    async fn get_template(
        &self,
        _request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn list_templates(
        &self,
        _request: Request<ListTemplatesMessage>,
    ) -> Result<Response<TemplateList>, Status> {
        todo!()
    }

    async fn delete_template(
        &self,
        _request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn create_template(
        &self,
        _request: Request<CreateTemplateMessage>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }
}
