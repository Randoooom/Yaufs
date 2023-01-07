resource "helm_release" "cert-manager" {
  name             = "cert-manager"
  namespace        = var.cert_manager_namespace
  create_namespace = true
  depends_on       = [helm_release.vault, null_resource.vault_setup]

  repository = "https://charts.jetstack.io"
  chart      = "cert-manager"

  set {
    name  = "prometheus.enabled"
    value = false
  }

  set {
    name  = "installCRDs"
    value = true
  }
}
