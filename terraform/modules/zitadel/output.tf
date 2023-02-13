output "nginx_private_client_id" {
  value = zitadel_application_oidc.nginx.client_id
}

output "nginx_private_client_secret" {
  value = zitadel_application_oidc.nginx.client_secret
}
