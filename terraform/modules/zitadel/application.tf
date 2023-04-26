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
  auth_method_type = "API_AUTH_METHOD_TYPE_PRIVATE_KEY_JWT"
}

resource "zitadel_application_key" "template_service" {
  depends_on = [zitadel_application_api.template_service]

  org_id     = zitadel_org.yaufs.id
  project_id = zitadel_project.yaufs_internal.id
  app_id     = zitadel_application_api.template_service.id
  key_type   = "KEY_TYPE_JSON"
  expiration_date = "2519-04-01T08:45:00Z"
}
resource "zitadel_machine_user" "template_service" {
  depends_on = [zitadel_org.yaufs]

  org_id    = zitadel_org.yaufs.id
  user_name = "template-service@${var.host}"
  name      = "template-service"
}

resource "zitadel_machine_key" "template_service" {
  depends_on = [zitadel_machine_user.template_service]

  org_id   = zitadel_org.yaufs.id
  user_id  = zitadel_machine_user.template_service.id
  key_type = "KEY_TYPE_JSON"
}

resource "zitadel_user_grant" "template_service" {
  depends_on = [
    zitadel_org.yaufs, zitadel_project.yaufs_internal, zitadel_machine_user.template_service,
  ]

  org_id     = zitadel_org.yaufs.id
  project_id = zitadel_project.yaufs_internal.id
  user_id    = zitadel_machine_user.template_service.id
  role_keys  = ["templating"]
}

################################################################

resource "zitadel_application_api" "control_plane" {
  depends_on = [zitadel_project.yaufs_internal]

  org_id           = zitadel_org.yaufs.id
  project_id       = zitadel_project.yaufs_internal.id
  name             = "control_plane"
  auth_method_type = "API_AUTH_METHOD_TYPE_PRIVATE_KEY_JWT"
}

resource "zitadel_application_key" "control_plane" {
  depends_on = [zitadel_application_api.control_plane]

  org_id     = zitadel_org.yaufs.id
  project_id = zitadel_project.yaufs_internal.id
  app_id     = zitadel_application_api.control_plane.id
  key_type   = "KEY_TYPE_JSON"
  expiration_date = "2519-04-01T08:45:00Z"
}
resource "zitadel_machine_user" "control_plane" {
  depends_on = [zitadel_org.yaufs]

  org_id    = zitadel_org.yaufs.id
  user_name = "control_plane@${var.host}"
  name      = "control_plane"
}

resource "zitadel_machine_key" "control_plane" {
  depends_on = [zitadel_machine_user.control_plane]

  org_id   = zitadel_org.yaufs.id
  user_id  = zitadel_machine_user.control_plane.id
  key_type = "KEY_TYPE_JSON"
}

resource "zitadel_user_grant" "control_plane" {
  depends_on = [
    zitadel_org.yaufs, zitadel_project.yaufs_internal, zitadel_machine_user.control_plane,
  ]

  org_id     = zitadel_org.yaufs.id
  project_id = zitadel_project.yaufs_internal.id
  user_id    = zitadel_machine_user.control_plane.id
  role_keys  = ["templating"]
}
