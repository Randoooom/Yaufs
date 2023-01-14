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

resource "kubernetes_service_account_v1" "vault_issuer" {
  metadata {
    name      = "vault-issuer"
    namespace = var.cert_manager_namespace
  }
}

resource "kubectl_manifest" "issuer_secret" {
  depends_on = [helm_release.cert-manager, kubernetes_service_account_v1.vault_issuer]
  yaml_body  = file("${path.module}/config/issuer-secret.yaml")
}

resource "kubectl_manifest" "issuer" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer_secret]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "ClusterIssuer"
    "metadata"   = {
      "name"      = "vault-issuer"
      "namespace" = var.cert_manager_namespace
    }
    "spec" = {
      "vault" = {
        "auth" = {
          "kubernetes" = {
            "mountPath" = "/v1/auth/kubernetes"
            "role"      = "vault-issuer"
            "secretRef" = {
              "key"  = "token"
              "name" = "issuer-token"
            }
          }
        }
        "caBundle" = data.local_file.vault_ca.content_base64
        "path"     = "pki/sign/cluster"
        "server"   = "https://vault.${var.vault_namespace}.svc.cluster.local:8200"
      }
    }
  })
}
