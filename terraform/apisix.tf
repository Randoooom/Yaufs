data "local_file" "vault_ca" {
  filename   = "${path.module}/output/vault.ca"
  depends_on = [null_resource.vault_init]
}

resource "kubectl_manifest" "certs" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "apisix"
      "namespace" = "default"
    }
    "spec" = {
      "commonName" = var.host
      "dnsNames"   = [
        var.host,
        "*.${var.host}",
      ]
      "issuerRef" = {
        "name" = "vault-issuer"
        "type" = "ClusterIssuer"
      }
      "secretName" = "apisix-tls"
    }
  })
}

resource "helm_release" "apisix" {
  name             = "apisix"
  namespace        = var.apisix_namespace
  depends_on       = [helm_release.cert-manager, helm_release.linkerd]
  create_namespace = true

  repository = "https://charts.apiseven.com"
  chart      = "apisix"

  values = [
    yamlencode({
      "extraVolumes" = [
        {
          name      = "yaufs-request-id"
          configMap = {
            name = "yaufs-request-id"
          }
        }
      ]
      "extraVolumeMounts" = [
        {
          name      = "yaufs-request-id"
          mountPath = "/plugins"
        }
      ]
      "apisix" = {
        "podAnnotations" = {
          "linkerd.io/inject" = "enabled"
        }
      }
      "gateway" = {
        "type" = "NodePort"
        "tls"  = {
          "enabled" = "true"
        }
      }
      "discovery" = {
        "enabled" = "true"
        "dns"     = {
          "servers" = [
            "10.97.152.16"
          ]
        }
      }
      "etcd" = {
        "replicaCount" = 1
      }
      "wasmPlugins" = {
        "enabled" = true
        "plugins" = [
          {
            "name" = "yaufs-request-id"
            "priority" = 2001
            "file" = "/plugins/yaufs_apisix_request_id.wasm"
            "http_request_phase" = "access"
          }
        ]
      }
      "plugins" = [
        "api-breaker",
        "public-api",
        "prometheus",
        "request-id",
        "gzip",
        "redirect",
        "openid-connect",
        "proxy-cache",
        "opentelemetry"
      ]
      "pluginAttrs" = {
        "prometheus" = {
          "enable_export_server" = false
          "export_uri"           = "/apisix/prometheus/metrics"
        }
        "opentelemetry" = {
          "trace_id_source" = "random"
          "resource"        = {
            "service.name" = "APISIX"
          }
          "collector" = {
            "address"         = "jaeger-default-collector-headless.${var.jaeger_namespace}.svc.cluster.local:4318"
            "request_timeout" = 3
          }
        }
      }
      "ingress-controller" = {
        "enabled" = true
        "config"  = {
          "apisix" = {
            "serviceNamespace" = var.apisix_namespace
            "adminAPIVersion"  = "v3"
          }
        }
        "podAnnotations" = {
          "linkerd.io/inject" = "ingress"
        }
      }
    })
  ]
}

resource "kubectl_manifest" "apisix_cluster" {
  depends_on = [helm_release.apisix]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixClusterConfig"
    "metadata"   = {
      "name" = "default"
    }
    "spec" = {
      "monitoring" = {
        "prometheus" = {
          "enable" = true
        }
      }
    }
  })
}

resource "kubectl_manifest" "apisix_metrics" {
  depends_on = [helm_release.apisix, kubectl_manifest.apisix_cluster]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixRoute"
    "metadata"   = {
      "name"      = "apisix-metrics"
      "namespace" = var.apisix_namespace
    }
    "spec" = {
      "http" = [
        {
          "backends" = [
            {
              "serviceName" = "apisix-admin",
              "servicePort" = 9180
            }
          ]
          "match" = {
            "hosts" = [
              "apisix-gateway.${var.apisix_namespace}.svc.cluster.local"
            ],
            "paths" = ["/apisix/prometheus/metrics"]
          },
          "name"    = "prometheus-public-api"
          "plugins" = [
            {
              "enable" = true
              "name"   = "public-api"
            },
          ]
        }
      ]
    }
  })
}

resource "kubectl_manifest" "tls" {
  depends_on = [helm_release.apisix]
  yaml_body  = yamlencode({
    "apiVersion" = "apisix.apache.org/v2"
    "kind"       = "ApisixTls"
    "metadata"   = {
      "name"      = "apisix"
      "namespace" = var.apisix_namespace
    }
    "spec" = {
      "hosts" = [
        var.host,
        "*.${var.host}",
      ]
      "secret" = {
        "name"      = "apisix-tls"
        "namespace" = "default"
      }
    }
  })
}

data "local_file" "vault_output" {
  depends_on = [null_resource.vault_setup]
  filename   = "${path.module}/output/output.json"
}

locals {
  vault_output_data = jsondecode(data.local_file.vault_output.content)
}

resource "kubectl_manifest" "apisix_openid" {
  depends_on = [helm_release.apisix, local.vault_output_data]
  yaml_body  = yamlencode(
    {
      "apiVersion" = "apisix.apache.org/v2"
      "kind"       = "ApisixPluginConfig"
      "metadata"   = {
        "name"      = "oidc"
        "namespace" = "prometheus"
      }
      "spec" = {
        "plugins" = [
          {
            "config" = {
              "client_id"     = local.vault_output_data.client_id
              "client_secret" = local.vault_output_data.client_secret
              "discovery"     = "http://vault.${var.vault_namespace}.svc.cluster.local:8200/v1/identity/oidc/provider/vault/.well-known/openid-configuration"
              "public_key"    = local.vault_output_data.public_key
              "realm"         = "apisix"
              "redirect_uri"  = "https://grafana.${var.host}/callback"
            }
            "enable" = true
            "name"   = "openid-connect"
          },
        ]
      }
    })
}
