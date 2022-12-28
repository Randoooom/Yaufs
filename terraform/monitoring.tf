resource "helm_release" "prometheus" {
  name             = "prometheus"
  namespace        = var.prometheus_namespace
  create_namespace = true

  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "kube-prometheus-stack"
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
              "serviceName" = "prometheus-grafana"
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
              "serviceName" = "prometheus-kube-prometheus-prometheus"
              "servicePort" = 9090
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
