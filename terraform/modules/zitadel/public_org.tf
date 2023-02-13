resource "zitadel_org" "public_yaufs" {
  name = var.zitadel_org
}

resource "zitadel_project" "public_yaufs" {
  depends_on = [zitadel_org.public_yaufs]

  name                     = var.zitadel_org
  org_id                   = zitadel_org.public_yaufs.id
  project_role_assertion   = true
  project_role_check       = true
  has_project_check        = true
  private_labeling_setting = "PRIVATE_LABELING_SETTING_ENFORCE_PROJECT_RESOURCE_OWNER_POLICY"
}

resource "zitadel_project_role" "public_admin" {
  depends_on = [zitadel_org.public_yaufs, zitadel_project.public_yaufs]

  org_id       = zitadel_org.public_yaufs.id
  project_id   = zitadel_project.public_yaufs.id
  role_key     = "admin"
  display_name = "Administration"
  group        = "Administration"
}

resource "zitadel_project_grant" "public_project_grant" {
  depends_on = [
    zitadel_org.public_yaufs, zitadel_project.public_yaufs, zitadel_org.public_yaufs, zitadel_project_role.public_admin
  ]

  org_id         = zitadel_org.public_yaufs.id
  project_id     = zitadel_project.public_yaufs.id
  granted_org_id = zitadel_org.public_yaufs.id
  role_keys      = [zitadel_project_role.public_admin.role_key]
}
