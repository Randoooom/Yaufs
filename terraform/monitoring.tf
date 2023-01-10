data "template_file" "prometheus_scrapes" {
  template = file("${path.module}/config/prometheus-scrapes.yaml")

  vars = {
    vault_namespace   = var.vault_namespace
    linkerd_namespace = var.linkerd_namespace
    jaeger_namespace  = var.jaeger_namespace
  }
}

resource "helm_release" "prometheus" {
  name             = "prometheus"
  namespace        = var.prometheus_namespace
  create_namespace = true

  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "prometheus"
  values     = [data.template_file.prometheus_scrapes.rendered]
}

resource "helm_release" "grafana" {
  name       = "grafana"
  namespace  = var.prometheus_namespace
  depends_on = [helm_release.prometheus, helm_release.loki]

  repository = "https://grafana.github.io/helm-charts"
  chart      = "grafana"

  values = [
    yamlencode({
      "datasources" = {
        "datasources.yaml" = {
          "apiVersion"  = 1
          "datasources" = [
            {
              "name" = "Prometheus"
              "type" = "prometheus"
              "url"  = "http://prometheus-server.${var.prometheus_namespace}.svc.cluster.local:80"
            },
            {
              "name" = "Jaeger"
              "type" = "jaeger"
              "url"  = "http://jaeger-default-query.${var.jaeger_namespace}.svc.cluster.local:16686"
            },
            {
              "name"   = "Loki"
              "type"   = "loki"
              "access" = "proxy"
              "url"    = "http://loki.loki.svc.cluster.local:3100"
            }
          ]
        }
      }
      "dashboardProviders" = {
        "dashboardproviders.yaml" = {
          "apiVersion" = 1
          "providers"  = [
            {
              "name"            = "default"
              "orgId"           = 1
              "folder"          = ""
              "type"            = "file"
              "disableDeletion" = true
              "editable"        = true
              "options"         = {
                "path" = "/var/lib/grafana/dashboards/default"
              }
            }
          ]
        }
      }
      "dashboards" = {
        "default" = {
          "apisix" = {
            "json" = <<EOF
              ${file("${path.module}/config/grafana/apisix.json")}
            EOF
          }
          "vault" = {
            "json" = <<EOF
              ${file("${path.module}/config/grafana/vault.json")}
            EOF
          }
        }
      }
    })
  ]
}

resource "helm_release" "loki" {
  name             = "loki"
  namespace        = "loki"
  depends_on       = [helm_release.prometheus]
  create_namespace = true

  repository = "https://grafana.github.io/helm-charts"
  chart      = "loki-stack"

  values = [
    yamlencode({
      "loki" = {
        "minio" = {
          "enabled" = true
        }
        "read" = {
          "replicas" = 1
        }
        "write" = {
          "replicas" = 1
        }
        "compactor" = {
          "retention_enabled" = true
        }
      }
    })
  ]
}

resource "kubectl_manifest" "monitoring_apisix" {
  depends_on = [helm_release.apisix, kubectl_manifest.apisix_openid]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixRoute"
    "metadata"   = {
      "name"      = "monitoring"
      "namespace" = var.prometheus_namespace
    }
    "spec" = {
      "http" = [
        {
          "backends" = [
            {
              "serviceName" = "grafana"
              "servicePort" = 80
            },
          ]
          "match" = {
            "hosts" = [
              "grafana.${var.host}",
            ]
            "paths" = [
              "/*",
            ]
          }
          "name"    = "grafana"
          "plugins" = [
            {
              "config" = {
                "http_to_https" = true
              }
              "enable" = true
              "name"   = "redirect"
            },
            {
              "config" = {
                "conf" = "|"
              }
              "name"     = "yaufs-request-id"
              "enable" = true
            },
            {
              "config" = {
                "sampler" = {
                  "name" = "always_on"
                }
              }
              "name" = "opentelemetry"
              "enable" = true
            }
          ],
          #                  "plugin_config_name" = "oidc"
        },
        {
          "backends" = [
            {
              "serviceName" = "prometheus-server"
              "servicePort" = 80
            },
          ]
          "match" = {
            "hosts" = [
              "prometheus.${var.host}",
            ]
            "paths" = [
              "/*",
            ]
          }
          "name"    = "prometheus"
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
