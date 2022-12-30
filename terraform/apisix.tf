resource "helm_release" "cert-manager" {
  name             = "cert-manager"
  namespace        = var.apisix_namespace
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

resource "kubectl_manifest" "issuer_secret" {
  depends_on = [helm_release.cert-manager]
  yaml_body  = file("${path.module}/config/issuer-secret.yaml")
}

resource "kubectl_manifest" "issuer" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer_secret]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Issuer"
    "metadata"   = {
      "name"      = "vault-issuer"
      "namespace" = "default"
    }
    "spec" = {
      "vault" = {
        "auth" = {
          "kubernetes" = {
            "mountPath" = "/v1/auth/kubernetes"
            "role"      = "issuer"
            "secretRef" = {
              "key"  = "token"
              "name" = "issuer-token-lmzpj"
            }
          }
        }
        "caBundle" = filebase64("${path.module}/output/vault.ca")
        "path"   = "pki/sign/apisix"
        "server" = "https://vault.${var.vault_namespace}.svc.cluster.local:8200"
      }
    }
  })
}

resource "time_sleep" "wait_for_issuer" {
  depends_on      = [kubectl_manifest.issuer]
  create_duration = "10s"
}

resource "kubectl_manifest" "certs" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, time_sleep.wait_for_issuer]
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
      }
      "secretName" = "apisix-tls"
    }
  })
}

resource "helm_release" "apisix" {
  name       = "apisix"
  namespace  = var.apisix_namespace
  depends_on = [helm_release.cert-manager]

  repository = "https://charts.apiseven.com"
  chart      = "apisix"

  set {
    name  = "ingress-controller.enabled"
    value = true
  }

  set {
    name  = "ingress-controller.config.apisix.serviceNamespace"
    value = "apisix"
  }

  set {
    name  = "gateway.type"
    value = "NodePort"
  }

  set {
    name  = "gateway.tls.enabled"
    value = "true"
  }

  set {
    name  = "discovery.enabled"
    value = true
  }

  set {
    name  = "discovery.consul_kv.servers"
    value = yamlencode(["consul.${var.consul_namespace}.svc.cluster.local:8501"])
  }

  set {
    name  = "discovery.consul_kv.prefix"
    value = "upstreams"
  }

  set {
    name  = "etcd.replicaCount"
    value = 1
  }

  set {
    name  = "pluginAttrs.prometheus.enable_export_server"
    value = false
  }

  set {
    name  = "pluginAttrs.prometheus.export_uri"
    value = "/apisix/prometheus/metrics"
  }

  set {
    name  = "plugins"
    value = "{api-breaker,public-api,prometheus,request-id,gzip,redirect,openid-connect,proxy-cache}"
  }

  #  set {
  #    name  = "serviceMonitor.enabled"
  #    value = true
  #  }
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
