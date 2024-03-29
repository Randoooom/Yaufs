output "nginx_private_client_id" {
  value = zitadel_application_oidc.nginx.client_id
}

output "nginx_private_client_secret" {
  value = zitadel_application_oidc.nginx.client_secret
}

output "template_service_service_account_key" {
  value = zitadel_machine_key.template_service.key_details
}

output "template_service_application_key_json" {
  value = zitadel_application_key.template_service.key_details
}

output "control_plane_service_account_key" {
  value = zitadel_machine_key.control_plane.key_details
}

output "control_plane_application_key_json" {
  value = zitadel_application_key.control_plane.key_details
}

output "project_id" {
  value = zitadel_project.yaufs_internal.id
}
