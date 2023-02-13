resource "kubernetes_namespace" "yaufs_template_service" {
  depends_on = [helm_release.linkerd, helm_release.jaeger_operator]

  metadata {
    name        = "template-service"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "helm_release" "yaufs_template_service" {
  name       = "template-service"
  namespace  = "template-service"
  depends_on = [
    kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver,
    kubernetes_namespace.yaufs_template_service
  ]

  chart = "${path.module}/../helm/yaufs-template-service"
}
