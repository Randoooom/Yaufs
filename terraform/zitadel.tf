resource "kubernetes_namespace" "zitadel" {
  depends_on = [helm_release.linkerd]

  metadata {
    name = "zitadel"
#    annotations = {
#      "linkerd.io/inject" = "enabled"
#    }
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
  depends_on = [kubernetes_namespace.zitadel, null_resource.vault_setup, helm_release.csi_driver]
  yaml_body  = yamlencode({
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
        secretPath: "zitadel/master-key"
        secretKey: "key"
      - objectName: "zitadel-config"
        secretPath: "zitadel/config"
        secretKey: "config"
      - objectName: "postgres-password"
        secretPath: "zitadel/postgres"
        secretKey: "password"
      - objectName: "postgres-repmgr-password"
        secretPath: "zitadel/postgres"
        secretKey: "repmgr-password"
      - objectName: "zitadel-postgres-password"
        secretPath: "zitadel/zitadel-postgres"
        secretKey: "password"
      - objectName: "zitadel-postgres-username"
        secretPath: "zitadel/zitadel-postgres"
        secretKey: "username"
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
        {
          "data" = [
            {
              "key"        = "repmgr-password"
              "objectName" = "postgres-repmgr-password"
            },
            {
              "key"        = "password"
              "objectName" = "postgres-password"
            }
          ]
          "secretName" = "postgres-credentials"
          "type"       = "Opaque"
        },
        {
          "data" = [
            {
              "key"        = "usernames"
              "objectName" = "zitadel-postgres-username"
            },
            {
              "key"        = "passwords"
              "objectName" = "zitadel-postgres-password"
            }
          ]
          "secretName" = "pgpool-users"
          "type"       = "Opaque"
        },
      ]
    }
  })
}

resource "kubectl_manifest" "zitadel_postgres_certificate" {
  depends_on = [helm_release.cert-manager, kubernetes_namespace.zitadel]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "postgres-tls"
      "namespace" = "zitadel"
    }
    "spec" = {
      "commonName" = "postgresql-postgresql.zitadel.svc.cluster.local"
      "dnsNames"   = [
        "postgresql-postgresql.zitadel.svc.cluster.local",
        "postgresql-postgresql-headless.zitadel.svc.cluster.local",
        "postgresql-pgpool.zitadel.svc.cluster.local"
      ]
      "issuerRef" = {
        "name" = "vault-issuer"
        "kind" = "ClusterIssuer"
      }
      "secretName" = "postgres-tls"
    }
  })
}

resource "helm_release" "zitadel_postgres" {
  name       = "postgresql-ha"
  namespace  = "zitadel"
  depends_on = [
    helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.zitadel, kubectl_manifest.zitadel_csi,
    kubectl_manifest.zitadel_postgres_certificate, helm_release.csi_driver
  ]

  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql-ha"

  values = [
    yamlencode({
      "fullnameOverride" = "postgresql"
      "postgresql"       = {
        "existingSecret" = "postgres-credentials"
        "tls"            = {
          "enabled"            = true
          "certificatesSecret" = "postgres-tls"
          "certFilename"       = "tls.crt"
          "certKeyFilename"    = "tls.key"
        }
        "extraVolumes" = [
          {
            "name" = "secrets-store"
            "csi"  = {
              "driver"           = "secrets-store.csi.k8s.io"
              "readOnly"         = true
              "volumeAttributes" = {
                "secretProviderClass" = "vault-zitadel"
              }
            }
          }
        ]
        "extraVolumeMounts" = [
          {
            "mountPath" = "/mnt/secrets-store"
            "name"      = "secrets-store"
            "readOnly"  = true
          }
        ]
      },
      "pgpool" = {
        "customUsersSecret" = "pgpool-users"
        "tls"               = {
          "enabled"            = true
          "certificatesSecret" = "postgres-tls"
          "certFilename"       = "tls.crt"
          "certKeyFilename"    = "tls.key"
        }
      }
      "serviceAccount" = {
        "create" = true
        "name"   = "postgres"
      }
    })
  ]
}

resource "helm_release" "zitadel" {
  name       = "zitadel"
  namespace  = "zitadel"
  depends_on = [
    kubectl_manifest.zitadel_csi, helm_release.zitadel_postgres,
  ]

  repository = "https://charts.zitadel.com"
  chart      = "zitadel"

  values = [
    yamlencode(
      {
        "zitadel" = {
          "masterkeySecretName" = "zitadel-master-key"
          "configSecretName"    = "zitadel-config"
          "configmapConfig"     = {
            "TLS" = {
              "Enabled" = false
            }
            "ExternalDomain" = "zitadel.${var.host}"
            "Database"       = {
              "postgres" = {
                "host"     = "postgresql-pgpool.zitadel.svc.cluster.local"
                "port"     = 5432
                "Database" = "zitadel"
                "Admin"    = {
                  "SSL" = {
                    "RootCert" = "/.secrets/ca.crt"
                  }
                }
                "User" = {
                  "SSL" = {
                    "RootCert" = "/.secrets/ca.crt"
                  }
                }
              }
            }
          }
          "dbSslRootCrtSecret"   = "postgres-tls"
          "dbSslClientCrtSecret" = "postgres-tls"
        }
      }
    )
  ]
}

#resource "kubectl_manifest" "yaufs_template_service_apisix" {
#  depends_on = [
#    helm_release.apisix, helm_release.zitadel
#  ]
#  yaml_body  = yamlencode({
#    "apiVersion" = "apisix.apache.org/v2"
#    "kind"       = "ApisixRoute"
#    "metadata"   = {
#      "name"      = "zitadel"
#      "namespace" = "zitadel"
#    }
#    "spec" = {
#      "http" = [
#        {
#          "backends" = [
#            {
#              "serviceName" = "yaufs-template-service"
#              "servicePort" = 8000
#            },
#          ]
#          "match" = {
#            "hosts" = [
#              "template.${var.host}",
#            ]
#            "paths" = [
#              "/*",
#            ]
#          }
#          "name"    = "yaufs-template-service"
#          "plugins" = [
#            {
#              "config" = {
#                "conf" = "|"
#              }
#              "name"   = "yaufs-request-id"
#              "enable" = true
#            },
#            {
#              "config" = {
#                "sampler" = {
#                  "name" = "always_on"
#                }
#                "additional_attributes" = ["route_id", "http_header"]
#                "additional_header_prefix_attributes" = ["x-request-id"]
#              }
#              "name"   = "opentelemetry"
#              "enable" = true
#            }
#          ],
#        },
#      ]
#    }
#  })
#}
