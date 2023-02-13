resource "zitadel_human_user" "zitadel_org_admin" {
  depends_on = [zitadel_org.public_yaufs]

  org_id            = zitadel_org.public_yaufs.id
  email             = "admin@${var.host}"
  is_email_verified = true
  user_name         = "admin@${var.host}"
  first_name        = "Admin"
  last_name         = var.zitadel_org
  display_name      = "Admin"
  initial_password  = var.zitadel_admin_password
}

resource "zitadel_org_member" "public_admin" {
  depends_on = [zitadel_org.public_yaufs, zitadel_human_user.zitadel_org_admin]

  org_id  = zitadel_org.public_yaufs.id
  user_id = zitadel_human_user.zitadel_org_admin.id
  roles   = ["ORG_OWNER"]
}

resource "zitadel_org_member" "internal_admin" {
  depends_on = [zitadel_org.yaufs_internal, zitadel_human_user.zitadel_org_admin]

  org_id  = zitadel_org.yaufs_internal.id
  user_id = zitadel_human_user.zitadel_org_admin.id
  roles   = ["ORG_OWNER"]
}

resource "zitadel_project_grant_member" "public_project_grant_admin" {
  depends_on = [zitadel_org.public_yaufs, zitadel_project.public_yaufs, zitadel_human_user.zitadel_org_admin]

  org_id     = zitadel_org.public_yaufs.id
  project_id = zitadel_project.public_yaufs.id
  grant_id   = zitadel_project_grant.public_project_grant.id
  user_id    = zitadel_human_user.zitadel_org_admin.id
  roles      = ["PROJECT_GRANT_OWNER"]
}

resource "zitadel_project_grant_member" "internal_project_grant_admin" {
  depends_on = [zitadel_org.yaufs_internal, zitadel_project.yaufs_internal, zitadel_human_user.zitadel_org_admin]

  org_id     = zitadel_org.yaufs_internal.id
  project_id = zitadel_project.yaufs_internal.id
  grant_id   = zitadel_project_grant.internal_project_grant.id
  user_id    = zitadel_human_user.zitadel_org_admin.id
  roles      = ["PROJECT_GRANT_OWNER"]
}

resource "zitadel_user_grant" "public_user_grant_admin" {
  depends_on = [
    zitadel_org.public_yaufs, zitadel_project.public_yaufs, zitadel_human_user.zitadel_org_admin,
    zitadel_project_grant_member.public_project_grant_admin
  ]

  org_id     = zitadel_org.public_yaufs.id
  project_id = zitadel_project.public_yaufs.id
  user_id    = zitadel_human_user.zitadel_org_admin.id
  role_keys  = ["admin"]
}

resource "zitadel_user_grant" "internal_user_grant_admin" {
  depends_on = [
    zitadel_org.yaufs_internal, zitadel_project.yaufs_internal, zitadel_human_user.zitadel_org_admin,
    zitadel_project_grant_member.internal_project_grant_admin
  ]

  org_id     = zitadel_org.yaufs_internal.id
  project_id = zitadel_project.yaufs_internal.id
  user_id    = zitadel_human_user.zitadel_org_admin.id
  role_keys  = ["admin"]
}

resource "zitadel_instance_member" "admin" {
  depends_on = [zitadel_human_user.zitadel_org_admin]

  user_id = zitadel_human_user.zitadel_org_admin.id
  roles   = ["IAM_OWNER"]
}