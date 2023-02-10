resource "kubernetes_namespace" "zitadel" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "zitadel"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubernetes_service_account" "zitadel_service_account" {
  depends_on = [kubernetes_namespace.zitadel]

  metadata {
    name      = "zitadel"
    namespace = "zitadel"

    annotations = {
      "helm.sh/hook"               = "pre-install,pre-upgrade"
      "helm.sh/hook-delete-policy" = "before-hook-creation"
      "helm.sh/hook-weight"        = "0"
    }
  }
}

resource "kubectl_manifest" "zitadel_csi" {
  depends_on = [
    kubernetes_namespace.zitadel, null_resource.vault_setup, helm_release.csi_driver
  ]
  yaml_body = yamlencode({
    "apiVersion" = "secrets-store.csi.x-k8s.io/v1"
    "kind"       = "SecretProviderClass"
    "metadata"   = {
      "name"      = "vault-zitadel"
      "namespace" = "zitadel"
    }
    "spec" = {
      "parameters" = {
        "objects"      = <<-EOT
      - objectName: "zitadel-master-key"
        secretPath: "zitadel/credentials"
        secretKey: "master-key"
      - objectName: "zitadel-config"
        secretPath: "zitadel/config"
        secretKey: "config"
      EOT
        "roleName"     = "zitadel"
        "vaultAddress" = "https://vault.vault.svc.cluster.local:8200"
      }
      "provider"      = "vault"
      "secretObjects" = [
        {
          "data" = [
            {
              "key"        = "masterkey"
              "objectName" = "zitadel-master-key"
            }
          ]
          "secretName" = "zitadel-master-key"
          "type"       = "Opaque"
        },
        {
          "data" = [
            {
              "key"        = "config-yaml"
              "objectName" = "zitadel-config"
            }
          ]
          "secretName" = "zitadel-config"
          "type"       = "Opaque"
        },
      ]
    }
  })
}

resource "helm_release" "zitadel_cockroach" {
  name       = "cockroachdb"
  namespace  = "zitadel"
  depends_on = [
    kubernetes_namespace.zitadel, kubectl_manifest.zitadel_csi,
    helm_release.csi_driver, helm_release.linkerd
  ]
  wait = false

  repository = "https://charts.cockroachdb.com/"
  chart      = "cockroachdb"

  values = [
    yamlencode({
      "tls" = {
        # zero-trust provided by linkerd, but for unkwnown reasons cockroach enforces tls in order to use authentication
        "enabled" = true
        "certs"   = {
          "selfSigner" = {
            "enabled" = false
          }
          "certManager"       = true
          "certManagerIssuer" = {
            "kind" = "ClusterIssuer"
            "name" = "vault-issuer"
          }
          "useCertManagerV1CRDs" = true
        }
      }
      "init" = {
        "annotations" = {
          "linkerd.io/inject" = "disabled"
        }
      }
      "statefulset" = {
        "annotations" = {
          "config.linkerd.io/default-inbound-policy" = "cluster-unauthenticated"
        }
      }
    })
  ]
}

resource "kubernetes_job" "zitadel_csi_mount" {
  depends_on = [
    kubectl_manifest.zitadel_csi, kubernetes_namespace.zitadel, kubernetes_service_account.zitadel_service_account,
    helm_release.zitadel_cockroach
  ]

  metadata {
    name      = "zitadel-csi-mount"
    namespace = "zitadel"
  }

  spec {
    template {
      metadata {
        annotations = {
          "config.linkerd.io/shutdown-grace-period" = "5"
        }
      }
      spec {
        service_account_name = "zitadel"

        container {
          name    = "zitadel-csi-mount"
          image   = "curlimages/curl:7.87.0"
          command = ["sh", "-c", "sleep 5; CODE=$?; curl -X POST http://localhost:4191/shutdown; exit $CODE;"]

          volume_mount {
            mount_path = "/mnt/secrets-store"
            name       = "secrets-store"
            read_only  = true
          }
        }

        restart_policy = "OnFailure"

        volume {
          name = "secrets-store"
          csi {
            driver            = "secrets-store.csi.k8s.io"
            read_only         = true
            volume_attributes = {
              "secretProviderClass" = "vault-zitadel"
            }
          }
        }
      }
    }
  }
}

resource "helm_release" "zitadel" {
  name       = "zitadel"
  namespace  = "zitadel"
  depends_on = [
    kubectl_manifest.zitadel_csi, helm_release.zitadel_cockroach, kubernetes_service_account.zitadel_service_account,
    kubernetes_job.zitadel_csi_mount, helm_release.prometheus
  ]

  repository = "https://charts.zitadel.com"
  chart      = "zitadel"

  values = [
    yamlencode(
      {
        "serviceAccount" = {
          "name" = "zitadel"
        }
        "zitadel" = {
          "masterkeySecretName" = "zitadel-master-key"
          "configSecretName"    = "zitadel-config"
          "configmapConfig"     = {
            "TLS" = {
              "Enabled" = false
            }
            "ExternalSecure" = true
            "ExternalPort"   = 443
            "ExternalDomain" = "auth.${var.host}"
            "Database"       = {
              "cockroach" = {
                "Host"  = "cockroachdb-public.zitadel.svc.cluster.local"
                "Admin" = {
                  "SSL" = {
                    "Mode" = "require"
                  }
                }
                "User" = {
                  "SSL" = {
                    "Mode" = "require"
                  }
                }
              }
            }
            "FirstInstance" = {
              "MachineKeyPath" = "/machinekey/zitadel-admin-sa.json"
              "Org"            = {
                "Machine" = {
                  "Machine" = {
                    "Username" = "zitadel-admin-sa"
                    "Name"     = "admin"
                  }
                  "MachineKey" = {
                    "Type" = 1
                  }
                }
              }
            }
          }
          "dbSslRootCrtSecret"   = "cockroachdb-root"
          "dbSslClientCrtSecret" = "cockroachdb-node"
        }
        "initJob" = {
          "podAnnotations" = {
            "config.linkerd.io/shutdown-grace-period" = "5"
          }
          "extraContainers" = [
            {
              "name"    = "linkerd-shutdown"
              "image"   = "curlimages/curl:7.87.0"
              "command" = [
                "sh", "-c",
                "sleep 30; CODE=$?; curl -X POST http://localhost:4191/shutdown; exit $CODE;"
              ]
            }
          ]
        }
        "setupJob" = {
          "podAnnotations" = {
            "config.linkerd.io/shutdown-grace-period" = "5"
          }
          "extraContainers" = [
            {
              "name"    = "linkerd-shutdown"
              "image"   = "curlimages/curl:7.87.0"
              "command" = [
                "sh", "-c",
                "sleep 30; CODE=$?; curl -X POST http://localhost:4191/shutdown; exit $CODE;"
              ]
            }
          ]
          "machinekeyWriterImage" = {
            "tag" = "1.25.6"
          }
        }
        "service" = {
          "annotations" = {
            "traefik.ingress.kubernetes.io/service.serversscheme" = "h2c"
          }
        }
        "metrics" = {
          "enabled"        = true
          "serviceMonitor" = {
            "enabled" = true
          }
        }
      }
    )
  ]
}

resource "kubectl_manifest" "zitadel_ingress" {
  depends_on = [kubernetes_namespace.zitadel, helm_release.traefik]

  yaml_body = yamlencode({
    "apiVersion" = "traefik.containo.us/v1alpha1"
    "kind"       = "IngressRoute"
    "metadata"   = {
      "name"      = "zitadel"
      "namespace" = "zitadel"
    }
    "spec" = {
      "entryPoints" = [
        "websecure",
      ]
      "routes" = [
        {
          "kind"     = "Rule"
          "match"    = "Host(`auth.${var.host}`)"
          "services" = [
            {
              "name"   = "zitadel"
              "port"   = 8080
              "scheme" = "h2c"
            },
          ]
        },
      ]
    }
  })
}

resource "null_resource" "zitadel_credentials" {
  depends_on = [helm_release.zitadel]

  provisioner "local-exec" {
    command = "chmod +x scripts/zitadel.sh; /bin/bash scripts/zitadel.sh"
  }
}

data "local_file" "zitadel_credentials" {
  filename   = "${path.module}/output/zitadel-admin.json"
  depends_on = [null_resource.zitadel_credentials]
}

provider "zitadel" {
  domain = "auth.dev.localhost"
  port   = "443"
  token  = "${path.module}/output/zitadel.json"
}

resource "zitadel_org" "yaufs" {
  depends_on = [data.local_file.zitadel_credentials]

  name = var.zitadel_org
}

resource "zitadel_human_user" "zitadel_org_admin" {
  depends_on = [zitadel_org.yaufs]

  org_id            = zitadel_org.yaufs.id
  email             = "admin@${var.host}"
  is_email_verified = true
  user_name         = "admin@${var.host}"
  first_name        = "Admin"
  last_name         = var.zitadel_org
  display_name      = "Admin"
  initial_password  = jsondecode(data.local_file.zitadel_credentials.content).password
}
