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
use yaufs_common::error::{Result, YaufsError};
use yaufs_common::yaufs_proto::fluvio::{TemplateCreated, TemplateDeleted, YaufsEvent};

pub async fn get_template(
    surreal: &Surreal<Client>,
    request: Request<TemplateId>,
) -> Result<Response<Template>> {
    let data = request.into_inner();

    // fetch the template
    let template: Option<Template> = sql_span!(surreal.select(("template", data.id)).await)?;

    Ok(Response::new(
        template.ok_or(YaufsError::NotFound("Template not found"))?,
    ))
}

pub async fn list_templates(
    surreal: &Surreal<Client>,
    request: Request<ListTemplatesRequest>,
) -> Result<Response<ListTemplatesResponse>> {
    let data = request.into_inner();

    // fetch a page of templates
    let templates = sql_span!(
        surreal
            .query("SELECT * FROM template WHERE name >= $page_token LIMIT $page_size")
            .bind(("page_token", &data.page_token))
            .bind(("page_size", &data.page_size + 1))
            .await
    )?
    .take::<Vec<Template>>(0)?;
    // take the last element out of the page list
    let (next_page_token, templates) = yaufs_common::next_page_token!(
        templates,
        data.page_size,
        |(template, slice)| Some((template.name.to_string(), slice.to_vec()))
    );

    Ok(Response::new(ListTemplatesResponse {
        templates,
        next_page_token,
    }))
}

pub async fn delete_template(
    surreal: &Surreal<Client>,
    producer: &Option<&TopicProducer>,
    request: Request<TemplateId>,
) -> Result<Response<Empty>> {
    let data = request.into_inner();

    sql_span!(surreal.delete(("template", &data.id)).await)?;
    // emit the event
    #[cfg(not(test))]
    producer
        .unwrap()
        .send(YaufsEvent::TEMPLATE_DELETED, TemplateDeleted::new(data.id))
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;

    Ok(Response::new(Empty {}))
}

pub async fn create_template(
    surreal: &Surreal<Client>,
    producer: &Option<&TopicProducer>,
    request: Request<CreateTemplateRequest>,
) -> Result<Response<Template>> {
    let data = request.into_inner();
    // create the template
    let template: Template = sql_span!(surreal.create("template").content(data).await)?;

    // emit the creation event
    #[cfg(not(test))]
    producer
        .unwrap()
        .send(
            YaufsEvent::TEMPLATE_CREATED,
            TemplateCreated::new(&template.id),
        )
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;

    Ok(Response::new(template))
}
