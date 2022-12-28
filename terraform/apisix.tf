resource "helm_release" "cert-manager" {
  name             = "cert-manager"
  namespace        = var.apisix_namespace
  create_namespace = true
  depends_on       = [helm_release.vault, helm_release.consul]

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
        "path"   = "pki/sign/apisix"
        "server" = "http://vault.${var.vault_namespace}.svc.cluster.local:8200"
      }
    }
  })
}

resource "time_sleep" "wait_for_issuer" {
  create_duration = "30s"
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
      "commonName" = "dev.localhost"
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
