resource "kubernetes_namespace" "vault" {
  metadata {
    name = "vault"
  }
}

resource "null_resource" "vault_init" {
  depends_on = [kubernetes_namespace.vault]

  provisioner "local-exec" {
    command = "chmod +x scripts/vault-init.sh; /bin/bash scripts/vault-init.sh"
  }
}

data "local_file" "vault_ca" {
  filename   = "${path.module}/output/vault.ca"
  depends_on = [null_resource.vault_init]
}

data "local_file" "vault_root" {
  filename   = "${path.module}/output/vault-root.ca"
  depends_on = [null_resource.vault_setup]
}

resource "helm_release" "vault" {
  name       = "vault"
  namespace  = "vault"
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
                disable_hostname          = true
                prometheus_retention_time = "1h"
              }

              service_registration "kubernetes" {}
EOF
          }
        }
        "ingress" = {
          "enabled"          = true
          "ingressClassName" = "nginx"
          "annotations"      = {
            "nginx.ingress.kubernetes.io/backend-protocol" = "HTTPS"
            "nginx.ingress.kubernetes.io/auth-url"         = "http://oauth2-proxy.nginx.svc.cluster.local:80/oauth2/auth"
            "nginx.ingress.kubernetes.io/auth-signin"      = "https://${var.host}/oauth2/start?rd=$scheme://$host$request_uri"
          }
          "hosts" = [
            {
              "host"  = "vault.${var.host}"
              "paths" = []
            }
          ]
        }
      }
      "injector" = {
        "enabled" = false
      }
      "csi" = {
        "enabled"   = true
        "extraArgs" = ["-vault-tls-ca-cert=/var/run/secrets/kubernetes.io/serviceaccount/ca.crt"]
      }
    }
    )
  ]
}

resource "null_resource" "vault_setup" {
  depends_on = [helm_release.vault]

  provisioner "local-exec" {
    command = "chmod +x scripts/setup.sh; chmod +x scripts/oidc.sh; /bin/bash scripts/setup.sh ${var.host}"
  }
}

resource "helm_release" "csi_driver" {
  depends_on = [null_resource.vault_setup]
  name       = "csi-driver"
  namespace  = "vault"

  repository = "https://kubernetes-sigs.github.io/secrets-store-csi-driver/charts"
  chart      = "secrets-store-csi-driver"

  set {
    name  = "syncSecret.enabled"
    value = true
  }
}
