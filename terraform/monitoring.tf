resource "kubernetes_namespace" "prometheus" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "prometheus"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubectl_manifest" "grafana_csi" {
  depends_on = [
    kubernetes_namespace.prometheus, null_resource.vault_setup, helm_release.csi_driver
  ]
  yaml_body = yamlencode({
    "apiVersion" = "secrets-store.csi.x-k8s.io/v1"
    "kind"       = "SecretProviderClass"
    "metadata"   = {
      "name"      = "vault-grafana"
      "namespace" = "prometheus"
    }
    "spec" = {
      "parameters" = {
        "objects"      = <<-EOT
      - objectName: "grafana-admin-username"
        secretPath: "monitoring/grafana/credentials"
        secretKey: "username"
      - objectName: "grafana-admin-password"
        secretPath: "monitoring/grafana/credentials"
        secretKey: "password"
      EOT
        "roleName"     = "grafana"
        "vaultAddress" = "https://vault.vault.svc.cluster.local:8200"
      }
      "provider"      = "vault"
      "secretObjects" = [
        {
          "data" = [
            {
              "key"        = "username"
              "objectName" = "grafana-admin-username"
            },
            {
              "key"        = "password"
              "objectName" = "grafana-admin-password"
            }
          ]
          "secretName" = "grafana"
          "type"       = "Opaque"
        },
      ]
    }
  })
}

resource "helm_release" "prometheus" {
  name       = "prometheus"
  namespace  = "prometheus"
  depends_on = [
    null_resource.vault_setup, helm_release.linkerd, kubernetes_namespace.prometheus, kubectl_manifest.grafana_csi,
    helm_release.loki
  ]

  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "kube-prometheus-stack"
  values     = [
    yamlencode({
      "grafana" = {
        "admin" = {
          "existingSecret" = "grafana"
          "userKey"        = "username"
          "passwordKey"    = "password"
        }
        "extraVolumeMounts" = [
          {
            "name"      = "secrets-store"
            "mountPath" = "/mnt/secrets-store"
            "csi"       = true
            "data"      = {
              "driver"           = "secrets-store.csi.k8s.io"
              "readOnly"         = true
              "volumeAttributes" = {
                "secretProviderClass" = "vault-grafana"
              }
            }
          }
        ]
        "additionalDataSources" = [
          {
            "name" = "Jaeger"
            "type" = "jaeger"
            "url"  = "http://jaeger-jaeger-operator-jaeger-query.jaeger.svc.cluster.local:16686"
          },
          {
            "name"   = "Loki"
            "type"   = "loki"
            "access" = "proxy"
            "url"    = "http://loki.loki.svc.cluster.local:3100"
          }
        ]
      }
      "alertmanager" = {
        "enabled" = false
      }
      "prometheusOperator" = {
        "admissionWebhooks" = {
          "enabled" = false
        }
        "tls" = {
          "enabled" = false
        }
      }
      "prometheus" = {
        "prometheusSpec" = {
          "additionalScrapeConfigs" = yamldecode(file("${path.module}/config/prometheus-scrapes.yaml"))
        }
      }
      "prometheus-node-exporter" = {
        "hostRootFsMount" = {
          "enabled" = false
        }
      }
    })
  ]
}

#resource "helm_release" "grafana" {
#  name       = "grafana"
#  namespace  = "prometheus"
#  depends_on = [helm_release.prometheus, helm_release.loki]
#
#  repository = "https://grafana.github.io/helm-charts"
#  chart      = "grafana"
#
#  values = [
#    yamlencode({
#      "datasources" = {
#        "datasources.yaml" = {
#          "apiVersion"  = 1
#          "datasources" = [
#            {
#              "name" = "Prometheus"
#              "type" = "prometheus"
#              "url"  = "http://prometheus-server.prometheus.svc.cluster.local:80"
#            },
#            {
#              "name" = "Jaeger"
#              "type" = "jaeger"
#              "url"  = "http://jaeger-jaeger-operator-jaeger-query.jaeger.svc.cluster.local:16686"
#            },
#            {
#              "name"   = "Loki"
#              "type"   = "loki"
#              "access" = "proxy"
#              "url"    = "http://loki.loki.svc.cluster.local:3100"
#            }
#          ]
#        }
#      }
#      "dashboardProviders" = {
#        "dashboardproviders.yaml" = {
#          "apiVersion" = 1
#          "providers"  = [
#            {
#              "name"            = "default"
#              "orgId"           = 1
#              "folder"          = ""
#              "type"            = "file"
#              "disableDeletion" = true
#              "editable"        = true
#              "options"         = {
#                "path" = "/var/lib/grafana/dashboards/default"
#              }
#            }
#          ]
#        }
#      }
#      "dashboards" = {
#        "default" = {
#          "vault" = {
#            "json" = <<EOF
#              ${file("${path.module}/config/grafana/vault.json")}
#            EOF
#          }
#        }
#      }
#    })
#  ]
#}

resource "kubernetes_namespace" "loki" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "loki"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "helm_release" "loki" {
  name       = "loki"
  namespace  = "loki"
  depends_on = [kubernetes_namespace.loki]

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

resource "kubectl_manifest" "monitoring_ingress" {
  depends_on = [helm_release.prometheus, helm_release.traefik]

  yaml_body = yamlencode({
    "apiVersion" = "traefik.containo.us/v1alpha1"
    "kind"       = "IngressRoute"
    "metadata"   = {
      "name"      = "monitoring"
      "namespace" = "prometheus"
    }
    "spec" = {
      "entryPoints" = [
        "websecure",
      ]
      "routes" = [
        {
          "kind"     = "Rule"
          "match"    = "Host(`grafana.${var.host}`)"
          "services" = [
            {
              "name"   = "prometheus-grafana"
              "port"   = 80
              "scheme" = "http"
            },
          ]
        },
      ]
    }
  })
}
