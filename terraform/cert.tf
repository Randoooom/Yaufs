resource "helm_release" "cert-manager" {
  name             = "cert-manager"
  namespace        = "cert-manager"
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

resource "kubernetes_service_account" "vault_issuer" {
  depends_on = [helm_release.cert-manager]

  metadata {
    name      = "vault-issuer"
    namespace = "cert-manager"
  }
}

resource "kubectl_manifest" "issuer_secret" {
  depends_on = [helm_release.cert-manager, kubernetes_service_account.vault_issuer]
  yaml_body  = file("${path.module}/config/issuer-secret.yaml")
}

resource "kubectl_manifest" "issuer" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer_secret]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "ClusterIssuer"
    "metadata"   = {
      "name"      = "vault-issuer"
      "namespace" = "cert-manager"
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
        "server"   = "https://vault.vault.svc.cluster.local:8200"
      }
    }
  })
}
