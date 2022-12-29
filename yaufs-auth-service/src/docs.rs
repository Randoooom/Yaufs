use aide::transform::TransformOpenApi;
use indexmap::IndexMap;

pub fn docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Auth-Service")
        .summary("Nice service")
        .description(include_str!("../README.md"))
        .security_scheme(
            "OpenId",
            aide::openapi::SecurityScheme::OpenIdConnect {
                description: None,
                open_id_connect_url: "".to_owned(),
                extensions: IndexMap::new(),
            },
        )
}
