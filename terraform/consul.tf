resource "helm_release" "consul" {
  name             = "consul"
  namespace        = var.consul_namespace
  create_namespace = true
  depends_on       = [helm_release.vault, null_resource.vault_setup]

  repository = "https://helm.releases.hashicorp.com"
  chart      = "consul"

  values = [
    yamlencode({
      "connectInject" = {
        "enabled"          = true
        "replicas"         = 1
        "transparentProxy" = {
          "defaultEnabled" = true
        }
      }
      "controller" = {
        "enabled" = true
      }
      "global" = {
        "acls" = {
          "bootstrapToken" = {
            "secretKey"  = "token"
            "secretName" = "consul/data/secret/bootstrap-token"
          }
          "manageSystemACLs" = true
        }
        "datacenter" = "dc1"
        "domain"     = "consul"
        "federation" = {
          "createFederationSecret" = false
          "enabled"                = false
        }
        "gossipEncryption" = {
          "secretKey"  = "gossip"
          "secretName" = "consul/data/secret/gossip"
        }
        "name"           = "consul"
        "secretsBackend" = {
          "vault" = {
            "agentAnnotations" = <<-EOT
        "vault.hashicorp.com/namespace": "admin"
        "vault.hashicorp.com/ca-cert": "/run/secrets/kubernetes.io/serviceaccount/ca.crt"
        EOT
            "connectCA"        = {
              "additionalConfig"    = "{\"connect\": [{ \"ca_config\": [{ \"namespace\": \"admin\"}]}]}"
              "address"             = "https://vault.${var.vault_namespace}.svc.cluster.local:8200"
              "intermediatePKIPath" = "connect-intermediate-dc1/"
              "rootPKIPath"         = "connect-root/"
            }
            "consulCARole"         = "consul-ca"
            "consulClientRole"     = "consul-client"
            "consulServerRole"     = "consul-server"
            "enabled"              = true
            "manageSystemACLsRole" = "consul-server"
          }
        }
        "tls" = {
          "caCert" = {
            "secretName" = "pki/cert/ca"
          }
          "enableAutoEncrypt" = true
          "enabled"           = true
        }
      }
      "metrics" = {
        "baseURL"  = "http://prometheus-server.${var.prometheus_namespace}.svc.cluster.local:80"
        "enabled"  = true
        "provider" = "prometheus"
      }
      "server" = {
        "exposeGossipAndRPCPorts" = true
        "replicas"                = 1
        "serverCert"              = {
          "secretName" = "pki/issue/consul-server"
        }
      }
      "syncCatalog" = {
        "consulNamespaces" = {
          "mirroringK8S" = true
        }
        "enabled"           = true
        "k8sDenyNamespaces" = [
          "kube-system",
          "kube-public",
          "consul",
        ]
      }
    })
  ]
}

resource "kubernetes_manifest" "consul_ingress" {
  depends_on = [helm_release.apisix, helm_release.consul]
  manifest   = {
    "apiVersion" = "networking.k8s.io/v1"
    "kind"       = "Ingress"
    "metadata"   = {
      "annotations" = {
        "k8s.apisix.apache.org/http-to-https"   = "true"
        "k8s.apisix.apache.org/upstream-scheme" = "https"
      }
      "name"      = "consul"
      "namespace" = var.consul_namespace
    }
    "spec" = {
      "ingressClassName" = "apisix"
      "rules"            = [
        {
          "host" = "consul.${var.host}"
          "http" = {
            "paths" = [
              {
                "backend" = {
                  "service" = {
                    "name" = "consul-server"
                    "port" = {
                      "number" = 8501
                    }
                  }
                }
                "path"     = "/*"
                "pathType" = "Prefix"
              },
              {
                "backend" = {
                  "service" = {
                    "name" = "consul-ui"
                    "port" = {
                      "number" = 443
                    }
                  }
                }
                "path"     = "/ui"
                "pathType" = "Prefix"
              },
            ]
          }
        },
      ]
    }
  }
}
