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
use fluvio::TopicProducer;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use template_service_v1_server::{TemplateServiceV1, TemplateServiceV1Server};

mod handler;

pub struct TemplateServiceV1Context {
    surreal: Surreal<Client>,
    producer: Option<TopicProducer>,
}

pub type Server = TemplateServiceV1Server<TemplateServiceV1Context>;

pub async fn new(surreal: Surreal<Client>) -> yaufs_common::error::Result<Server> {
    #[cfg(not(test))]
    let producer = Some(yaufs_common::fluvio_util::producer().await?);
    #[cfg(test)]
    let producer = None;

    Ok(TemplateServiceV1Server::new(TemplateServiceV1Context {
        surreal,
        producer,
    }))
}

#[async_trait]
impl TemplateServiceV1 for TemplateServiceV1Context {
    #[instrument(skip_all)]
    async fn get_template(
        &self,
        request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        let result = handler::get_template(&self.surreal, request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn list_templates(
        &self,
        request: Request<ListTemplatesRequest>,
    ) -> Result<Response<ListTemplatesResponse>, Status> {
        let result = handler::list_templates(&self.surreal, request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn delete_template(
        &self,
        request: Request<TemplateId>,
    ) -> Result<Response<Empty>, Status> {
        let result =
            handler::delete_template(&self.surreal, &self.producer.as_ref(), request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn create_template(
        &self,
        request: Request<CreateTemplateRequest>,
    ) -> Result<Response<Template>, Status> {
        let result =
            handler::create_template(&self.surreal, &self.producer.as_ref(), request).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::init;
    use surrealdb::sql;
    use tonic::Response;
    use yaufs_common::yaufs_proto::template_service_v1::template_service_v1_client::TemplateServiceV1Client;
    use yaufs_common::yaufs_proto::template_service_v1::{
        CreateTemplateRequest, ListTemplatesRequest, ListTemplatesResponse,
    };

    #[tokio::test]
    async fn test_create_template() -> Result<(), Box<dyn std::error::Error>> {
        let (address, _, _) = init().await?;

        let mut client = TemplateServiceV1Client::connect(address).await?;
        let request = tonic::Request::new(CreateTemplateRequest {
            name: "test-name".to_string(),
            image: "test-image".to_string(),
        });
        client.create_template(request).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_list_templates() -> Result<(), Box<dyn std::error::Error>> {
        let (address, surreal, _) = init().await?;

        surreal
            .query(sql! (CREATE template SET image = $image1, name = $name1))
            .query(sql! (CREATE template SET image = $image2, name = $name2))
            .bind(("image1", "test1"))
            .bind(("image2", "test2"))
            .bind(("name1", "test1"))
            .bind(("name2", "test2"))
            .await?;
        let mut client = TemplateServiceV1Client::connect(address).await?;
        let request = tonic::Request::new(ListTemplatesRequest {
            page_size: 10,
            page_token: "".to_owned(),
        });

        let response: Response<ListTemplatesResponse> = client.list_templates(request).await?;
        let data = response.into_inner();
        assert_eq!(data.templates.len(), 2);

        Ok(())
    }
}
