resource "zitadel_org" "yaufs" {
  name = var.zitadel_org
}

resource "zitadel_project" "yaufs" {
  depends_on = [zitadel_org.yaufs]

  name                   = var.zitadel_org
  org_id                 = zitadel_org.yaufs.id
  project_role_assertion = true
  project_role_check     = true
  has_project_check      = true
}

resource "zitadel_project_role" "admin" {
  depends_on = [zitadel_org.yaufs, zitadel_project.yaufs]

  org_id       = zitadel_org.yaufs.id
  project_id   = zitadel_project.yaufs.id
  role_key     = "admin"
  display_name = "Administration"
  group        = "Administration"
}

resource zitadel_project_grant project_grant {
  depends_on = [zitadel_org.yaufs, zitadel_project.yaufs, zitadel_org.yaufs, zitadel_project_role.admin]

  org_id         = zitadel_org.yaufs.id
  project_id     = zitadel_project.yaufs.id
  granted_org_id = zitadel_org.yaufs.id
  role_keys      = [zitadel_project_role.admin.role_key]
}
