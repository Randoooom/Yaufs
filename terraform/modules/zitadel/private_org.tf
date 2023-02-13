resource "zitadel_org" "yaufs_internal" {
  name = "internal"
}

resource "zitadel_project" "yaufs_internal" {
  depends_on = [zitadel_org.public_yaufs]

  name                     = "internal"
  org_id                   = zitadel_org.yaufs_internal.id
  project_role_assertion   = true
  project_role_check       = true
  has_project_check        = true
  private_labeling_setting = "PRIVATE_LABELING_SETTING_ENFORCE_PROJECT_RESOURCE_OWNER_POLICY"
}

resource "zitadel_project_role" "internal_admin" {
  depends_on = [zitadel_org.yaufs_internal, zitadel_project.yaufs_internal]

  org_id       = zitadel_org.yaufs_internal.id
  project_id   = zitadel_project.yaufs_internal.id
  role_key     = "admin"
  display_name = "Administration"
  group        = "Administration"
}

resource "zitadel_project_grant" "internal_project_grant" {
  depends_on = [zitadel_project.yaufs_internal, zitadel_org.yaufs_internal, zitadel_project_role.internal_admin]

  org_id         = zitadel_org.yaufs_internal.id
  project_id     = zitadel_project.yaufs_internal.id
  granted_org_id = zitadel_org.yaufs_internal.id
  role_keys      = [zitadel_project_role.internal_admin.role_key]
}
