resource "zitadel_human_user" "zitadel_org_admin" {
  depends_on = [zitadel_org.yaufs]

  org_id            = zitadel_org.yaufs.id
  email             = "admin@${var.host}"
  is_email_verified = true
  user_name         = "admin@${var.host}"
  first_name        = "Admin"
  last_name         = var.zitadel_org
  display_name      = "Admin"
  initial_password  = var.zitadel_admin_password
}

resource "zitadel_project_grant_member" "project_grant_admin" {
  depends_on = [zitadel_org.yaufs, zitadel_project.yaufs, zitadel_human_user.zitadel_org_admin]

  org_id     = zitadel_org.yaufs.id
  project_id = zitadel_project.yaufs.id
  grant_id   = zitadel_project_grant.project_grant.id
  user_id    = zitadel_human_user.zitadel_org_admin.id
  roles      = ["PROJECT_GRANT_OWNER"]
}