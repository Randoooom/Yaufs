resource "helm_release" "prometheus" {
  name             = "prometheus"
  namespace        = var.prometheus_namespace
  create_namespace = true

  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "prometheus"
  values     = [file("${path.module}/config/prometheus-scrapes.yaml")]
}

resource "helm_release" "grafana" {
  name       = "grafana"
  namespace  = var.prometheus_namespace
  depends_on = [helm_release.prometheus]

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
            }
          ]
        }
      }
    })
  ]
}

resource "kubectl_manifest" "monitoring_apisix" {
  depends_on = [helm_release.apisix]
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
                  ]
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
