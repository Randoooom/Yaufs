resource "helm_release" "vault" {
  name             = "vault"
  namespace        = var.vault_namespace
  create_namespace = true

  repository = "https://helm.releases.hashicorp.com"
  chart      = "vault"

  set {
    name  = "injector.enabled"
    value = true
  }
}

resource "kubectl_manifest" "vault_apisix" {
  depends_on = [helm_release.apisix, helm_release.vault]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixRoute"
    "metadata"   = {
      "name"      = "vault"
      "namespace" = var.vault_namespace
    }
    "spec" = {
      "http" = [
        {
          "backends" = [
            {
              "serviceName" = "vault"
              "servicePort" = 8200
            },
          ]
          "match" = {
            "hosts" = [
              "vault.${var.host}",
            ]
            "paths" = [
              "/*",
            ]
          }
          "name"    = "vault"
          "plugins" = [
            {
              "config" = {
                "http_to_https" = true
              }
              "enable" = true
              "name"   = "redirect"
            },
          ]
        },
      ]
    }
  })
}
