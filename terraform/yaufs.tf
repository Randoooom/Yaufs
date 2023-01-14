resource "helm_release" "yaufs_template_service" {
  name             = "template-service"
  namespace        = "template-service"
  depends_on       = [kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver]
  create_namespace = true

  chart = "${path.module}/../helm/yaufs-template-service"
}
