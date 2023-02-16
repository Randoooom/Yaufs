locals {
  redirect_uris             = ["https://${var.host}/oauth2/callback"]
  post_logout_redirect_uris = ["https://${var.host}/oauth2/sign_out/"]
}

resource "zitadel_application_oidc" "nginx" {
  depends_on = [zitadel_project.yaufs_internal]

  project_id = zitadel_project.yaufs_internal.id
  org_id     = zitadel_org.yaufs.id

  name                        = "nginx"
  redirect_uris               = local.redirect_uris
  response_types              = ["OIDC_RESPONSE_TYPE_CODE"]
  grant_types                 = ["OIDC_GRANT_TYPE_AUTHORIZATION_CODE"]
  post_logout_redirect_uris   = local.post_logout_redirect_uris
  app_type                    = "OIDC_APP_TYPE_WEB"
  auth_method_type            = "OIDC_AUTH_METHOD_TYPE_BASIC"
  version                     = "OIDC_VERSION_1_0"
  clock_skew                  = "0s"
  access_token_type           = "OIDC_TOKEN_TYPE_BEARER"
  access_token_role_assertion = true
  id_token_role_assertion     = true
  id_token_userinfo_assertion = false
}

################################################################

resource "zitadel_application_api" "template_service" {
  depends_on = [zitadel_project.yaufs_internal]

  org_id           = zitadel_org.yaufs.id
  project_id       = zitadel_project.yaufs_internal.id
  name             = "template-service"
  auth_method_type = "API_AUTH_METHOD_TYPE_BASIC"
}
