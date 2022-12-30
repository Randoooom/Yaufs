resource "null_resource" "vault_init" {
  provisioner "local-exec" {
    command = "chmod +x scripts/cert.sh; /bin/bash scripts/cert.sh ${var.vault_namespace}"
  }
}

resource "helm_release" "vault" {
  name       = "vault"
  namespace  = var.vault_namespace
  depends_on = [null_resource.vault_init]

  repository = "https://helm.releases.hashicorp.com"
  chart      = "vault"

  values = [
    yamlencode({
      "global" = {
        "enabled"    = true
        "tlsDisable" = false
      }
      "server" = {
        "annotations" = {
          "consul.hashicorp.com/connect-inject"       = "true"
          "consul.hashicorp.com/connect-service"      = "vault"
          "consul.hashicorp.com/connect-service-port" = "8200"
          "consul.hashicorp.com/transparent-proxy"    = "false"
        }
        "extraEnvironmentVars" = {
          "VAULT_CACERT"  = "/vault/userconfig/vault-ha-tls/vault.ca"
          "VAULT_TLSCERT" = "/vault/userconfig/vault-ha-tls/vault.crt"
          "VAULT_TLSKEY"  = "/vault/userconfig/vault-ha-tls/vault.key"
          "VAULT_ADDR"    = "https://127.0.0.1:8200"
        }
        "volumes" = [
          {
            "name"   = "userconfig-vault-ha-tls"
            "secret" = {
              "defaultMode" = 420
              "secretName"  = "vault-ha-tls"
            }
          }
        ]
        "volumeMounts" = [
          {
            "mountPath" = "/vault/userconfig/vault-ha-tls"
            "name"      = "userconfig-vault-ha-tls"
            "readOnly"  = true
          }
        ]
        "standalone" = {
          "enabled" = false
        }
        "affinity" = ""
        "ha"       = {
          "enabled"   = true
          "replicas"  = 3
          "setNodeId" = true
          "raft"      = {
            "enabled" = true
            "config"  = <<EOF
              ui = true

              listener "tcp" {
                tls_disable = 0
                address = "[::]:8200"
                cluster_address = "[::]:8201"
                tls_cert_file = "/vault/userconfig/vault-ha-tls/vault.crt"
                tls_key_file  = "/vault/userconfig/vault-ha-tls/vault.key"
                tls_client_ca_file = "/vault/userconfig/vault-ha-tls/vault.ca"
              }

              disable_mlock = true

              storage "raft" {
                path = "/vault/data"
              }

              telemetry {
                disable_hostname = true
                prometheus_retention_time = "1h"
              }

              service_registration "consul" {
                address      = "https://consul-server.${var.consul_namespace}.svc.cluster.local:8501"
              }
EOF
          }
        }
      }
      "injector" = {
        "enabled" = true
      }
    }
    )
  ]
}

resource "null_resource" "vault_setup" {
  depends_on = [helm_release.vault]

  provisioner "local-exec" {
    command = "chmod +x scripts/setup.sh; chmod +x scripts/oidc.sh; /bin/bash scripts/setup.sh ${var.host} ${var.vault_namespace} ${var.consul_namespace}"
  }
}

resource "kubernetes_manifest" "vault_ingress" {
  depends_on = [helm_release.apisix, helm_release.vault]
  manifest   = {
    "apiVersion" = "networking.k8s.io/v1"
    "kind"       = "Ingress"
    "metadata"   = {
      "annotations" = {
        "k8s.apisix.apache.org/http-to-https"   = "true"
        "k8s.apisix.apache.org/upstream-scheme" = "https"
      }
      "name"      = "vault"
      "namespace" = var.vault_namespace
    }
    "spec" = {
      "ingressClassName" = "apisix"
      "rules"            = [
        {
          "host" = "vault.${var.host}"
          "http" = {
            "paths" = [
              {
                "backend" = {
                  "service" = {
                    "name" = "vault"
                    "port" = {
                      "number" = 8200
                    }
                  }
                }
                "path"     = "/*"
                "pathType" = "Prefix"
              }
            ]
          }
        },
      ]
    }
  }
}
