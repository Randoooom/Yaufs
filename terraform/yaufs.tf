resource "kubernetes_namespace" "yaufs_control_plane" {
  depends_on = [helm_release.linkerd, helm_release.jaeger_operator]

  metadata {
    name        = "control-plane"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubernetes_secret" "control_plane_key" {
  metadata {
    name      = "oidc-keys"
    namespace = kubernetes_namespace.yaufs_control_plane.metadata[0].name
  }

  data = {
    "service-account-json" = module.zitadel.control_plane_service_account_key
    "application-json"     = module.zitadel.control_plane_application_key_json
  }
}

resource "kubernetes_secret" "control_plane_ca" {
  metadata {
    name      = "vault-ca"
    namespace = kubernetes_namespace.yaufs_control_plane.metadata[0].name
  }

  data = {
    "vault.crt" = data.local_file.vault_root.content
  }
}

resource "kubernetes_secret" "control_plane_fluvio" {
  depends_on = [null_resource.fluvio_setup, data.kubernetes_secret.fluvio]

  metadata {
    name      = "fluvio-tls"
    namespace = kubernetes_namespace.yaufs_control_plane.metadata[0].name
  }

  data = {
    "ca.crt"  = data.kubernetes_secret.fluvio.data["ca.crt"]
    "tls.crt" = data.kubernetes_secret.fluvio.data["tls.crt"]
    "tls.key" = data.kubernetes_secret.fluvio.data["tls.key"]
  }
}


resource "helm_release" "yaufs_control_plane" {
  name       = "control-plane"
  namespace  = kubernetes_namespace.yaufs_control_plane.metadata[0].name
  depends_on = [
    kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver,
    kubernetes_namespace.yaufs_control_plane, data.kubernetes_service.nginx, kubernetes_secret.control_plane_ca,
    kubernetes_secret.control_plane_fluvio
  ]

  chart = "${path.module}/../helm/yaufs-control-plane"

  values = [
    yamlencode({
      "logLevel" = "Info",
      "oidc"     = {
        "issuer"    = "https://auth.${var.host}"
        "hostAlias" = {
          "enabled"  = true
          "ip"       = data.kubernetes_service.nginx.spec.0.cluster_ip
          "hostname" = "auth.${var.host}"
        }
        "projectId" = tostring(module.zitadel.project_id)
        "caSecret" = "vault-ca"
      }
    })
  ]
}

resource "kubernetes_namespace" "yaufs_template_service" {
  depends_on = [
    helm_release.linkerd, helm_release.jaeger_operator
  ]

  metadata {
    name        = "template-service"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubernetes_secret" "template_service_keys" {
  metadata {
    name      = "oidc-keys"
    namespace = kubernetes_namespace.yaufs_template_service.metadata[0].name
  }

  data = {
    "service-account-json" = module.zitadel.template_service_service_account_key
    "application-json"     = module.zitadel.template_service_application_key_json
  }
}

resource "kubernetes_secret" "template_service_ca" {
  metadata {
    name      = "vault-ca"
    namespace = kubernetes_namespace.yaufs_template_service.metadata[0].name
  }

  data = {
    "vault.crt" = data.local_file.vault_root.content
  }
}

resource "kubernetes_secret" "template_service_fluvio" {
  depends_on = [null_resource.fluvio_setup, data.kubernetes_secret.fluvio]

  metadata {
    name      = "fluvio-tls"
    namespace = kubernetes_namespace.yaufs_template_service.metadata[0].name
  }

  data = {
    "ca.crt"  = data.kubernetes_secret.fluvio.data["ca.crt"]
    "tls.crt" = data.kubernetes_secret.fluvio.data["tls.crt"]
    "tls.key" = data.kubernetes_secret.fluvio.data["tls.key"]
  }
}

resource "helm_release" "yaufs_template_service" {
  name       = "template-service"
  namespace  = "template-service"
  depends_on = [
    kubectl_manifest.issuer, null_resource.vault_setup, helm_release.csi_driver,
    kubernetes_namespace.yaufs_template_service, data.kubernetes_service.nginx, kubernetes_secret.template_service_ca,
    kubernetes_secret.template_service_fluvio
  ]

  chart = "${path.module}/../helm/yaufs-template-service"

  values = [
    yamlencode({
      "logLevel" = "Info",
      "oidc"     = {
        "issuer"    = "https://auth.${var.host}"
        "hostAlias" = {
          "enabled"  = true
          "ip"       = data.kubernetes_service.nginx.spec.0.cluster_ip
          "hostname" = "auth.${var.host}"
        }
        "projectId" = tostring(module.zitadel.project_id)
        "caSecret" = "vault-ca"
      }
    })
  ]
}
