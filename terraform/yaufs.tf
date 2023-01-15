resource "helm_release" "yaufs_template_service" {
  name             = "template-service"
  namespace        = "template-service"
  depends_on       = [kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver]
  create_namespace = true

  chart = "${path.module}/../helm/yaufs-template-service"
}

resource "kubectl_manifest" "yaufs_template_service_upstream" {
  depends_on = [helm_release.apisix, helm_release.yaufs_template_service]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixUpstream"
    "metadata"   = {
      "name"      = "yaufs-template-service"
      "namespace" = "template-service"
    }
    "spec" = {
      "scheme" = "grpc"
    }
  })
}

resource "kubectl_manifest" "yaufs_template_service_apisix" {
  depends_on = [
    helm_release.apisix, helm_release.yaufs_template_service, kubectl_manifest.yaufs_template_service_upstream
  ]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixRoute"
    "metadata"   = {
      "name"      = "template-service"
      "namespace" = "template-service"
    }
    "spec" = {
      "http" = [
        {
          "backends" = [
            {
              "serviceName" = "yaufs-template-service"
              "servicePort" = 8000
            },
          ]
          "match" = {
            "hosts" = [
              "template.${var.host}",
            ]
            "paths" = [
              "/*",
            ]
          }
          "name"    = "yaufs-template-service"
          "plugins" = [
            {
              "config" = {
                "conf" = "|"
              }
              "name"   = "yaufs-request-id"
              "enable" = true
            },
            {
              "config" = {
                "sampler" = {
                  "name" = "always_on"
                }
                "additional_attributes" = ["route_id", "http_header"]
                "additional_header_prefix_attributes" = ["x-request-id"]
              }
              "name"   = "opentelemetry"
              "enable" = true
            }
          ],
        },
      ]
    }
  })
}
