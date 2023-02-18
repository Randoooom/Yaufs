resource "kubernetes_namespace" "yaufs_template_service" {
  depends_on = [helm_release.linkerd, helm_release.jaeger_operator]

  metadata {
    name        = "template-service"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubernetes_secret" "template_service_keys" {
  metadata {
    name      = "oidc-keys"
    namespace = kubernetes_namespace.yaufs_template_service.metadata[0].name
  }

  data = {
    "service-account-json" = module.zitadel.template_service_service_account_key
    "application-json"     = module.zitadel.template_service_application_key_json
  }
}

resource "kubernetes_secret" "template_service_ca" {
  metadata {
    name      = "vault-ca"
    namespace = kubernetes_namespace.yaufs_template_service.metadata[0].name
  }

  data = {
    "vault.crt" = data.local_file.vault_root.content
  }
}

resource "helm_release" "yaufs_template_service" {
  name       = "template-service"
  namespace  = "template-service"
  depends_on = [
    kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver,
    kubernetes_namespace.yaufs_template_service, data.kubernetes_service.nginx, kubernetes_secret.template_service_ca
  ]

  chart = "${path.module}/../helm/yaufs-template-service"

  values = [
    yamlencode({
      "logLevel" = "Debug",
      "oidc"     = {
        "issuer"    = "https://auth.${var.host}"
        "hostAlias" = {
          "enabled"  = true
          "ip"       = data.kubernetes_service.nginx.spec.0.cluster_ip
          "hostname" = "auth.${var.host}"
        }
        "caSecret" = "vault-ca"
      }
    })
  ]
}
