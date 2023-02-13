resource "kubernetes_namespace" "nginx" {
  depends_on = [helm_release.linkerd]

  metadata {
    name = "nginx"
  }
}

resource "kubectl_manifest" "certs" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.nginx]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "nginx"
      "namespace" = kubernetes_namespace.nginx.metadata[0].name
    }
    "spec" = {
      "commonName" = var.host
      "dnsNames"   = [
        var.host,
        "*.${var.host}",
      ]
      "issuerRef" = {
        "name" = "vault-issuer"
        "kind" = "ClusterIssuer"
      }
      "secretName" = "nginx-tls"
    }
  })
}

resource "helm_release" "nginx" {
  name       = "ingress-nginx"
  namespace  = kubernetes_namespace.nginx.metadata[0].name
  depends_on = [
    kubernetes_namespace.nginx, kubectl_manifest.certs, helm_release.jaeger_operator
  ]

  repository = "https://kubernetes.github.io/ingress-nginx"
  chart      = "ingress-nginx"

  values = [
    yamlencode({
      "controller" = {
        "podAnnotations" = {
          "linkerd.io/inject" = "enabled"
        }
        "annotations" = {
          "sidecar.jaegertracing.io/inject" = "true"
        }
        "extraArgs" = {
          "default-ssl-certificate" = "nginx/nginx-tls"
        }
        "service" = {
          "enableHttp" = false
        }
        "metrics" = {
          "serviceMonitor" = {
            "enabled" = true
          }
        }
        "config" = {
          enable-opentracing = "true"
          jaeger-endpoint    = "http://localhost:5778/sampling"
        }
        "admissionWebhooks" = {
          "enabled" = false
        }
      }
    })
  ]
}

resource "random_password" "oauth2_proxy_cookie_secret" {
  length  = 32
  special = false
}

resource "kubernetes_secret" "oauth2_proxy_vault_ca" {
  depends_on = [data.local_file.vault_root]

  metadata {
    name      = "vault-ca"
    namespace = kubernetes_namespace.nginx.metadata[0].name
  }

  data = {
    "vault.ca" = data.local_file.vault_root.content
  }
}

resource "helm_release" "oauth2_proxy" {
  name       = "oauth2-proxy"
  namespace  = kubernetes_namespace.nginx.metadata[0].name
  depends_on = [kubernetes_namespace.nginx, module.zitadel, kubernetes_secret.oauth2_proxy_vault_ca]

  repository = "https://oauth2-proxy.github.io/manifests"
  chart      = "oauth2-proxy"

  values = [
    yamlencode({
      "hostAlias" = {
        "enabled"  = true
        "ip"       = "10.43.154.247"
        "hostname" = "auth.${var.host}"
      }
      "podAnnotations" = {
        "linkerd.io/inject" = "enabled"
      }
      "extraVolumes" = [
        {
          "name"   = "ca-bundle"
          "secret" = {
            "secretName" = "vault-ca"
          }
        }
      ]
      "extraVolumeMounts" = [
        {
          "mountPath" = "/etc/ssl/certs"
          "name"      = "ca-bundle"
        }
      ]
      "config" = {
        "clientID"     = module.zitadel.nginx_private_client_id
        "clientSecret" = module.zitadel.nginx_private_client_secret
        "cookieSecret" = random_password.oauth2_proxy_cookie_secret.result
        "configFile"   = <<EOF
provider = "oidc"
user_id_claim = "sub"
provider_display_name = "ZITADEL"
redirect_url = "https://${var.host}/oauth2/callback"
oidc_issuer_url = "https://auth.${var.host}"
upstreams = [
    "https://monitoring.${var.host}"
]
email_domains = [
    "*"
]
cookie_domains = [
    ".${var.host}",
    "${var.host}"
]
whitelist_domains = [
    ".${var.host}",
    "${var.host}"
]
pass_access_token = true
skip_provider_button = true
EOF
      }
      "ingress" = {
        "enabled"   = true
        "className" = "nginx"
        "hosts"     = [var.host]
        "path"      = "/oauth2"
      }
    })
  ]
}
