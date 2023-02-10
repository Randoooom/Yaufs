resource "kubernetes_namespace" "traefik" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "traefik"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubectl_manifest" "certs" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.traefik]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "traefik"
      "namespace" = "traefik"
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
      "secretName" = "traefik-tls"
    }
  })
}

resource "helm_release" "traefik" {
  name       = "traefik"
  namespace  = "traefik"
  depends_on = [
    kubernetes_namespace.traefik, kubectl_manifest.certs, helm_release.jaeger_operator
  ]
  wait = false

  repository = "https://traefik.github.io/charts"
  chart      = "traefik"

  values = [
    yamlencode({
      "experimental" = {
        "v3" = {
          "enabled" = true
        }
      }
      "image" = {
        "tag" = "v3.0"
      }
      "deployment" = {
        "annotations" = {
          "sidecar.jaegertracing.io/inject" = "true"
        }
      }
      "tlsStore" = {
        "default" = {
          "defaultCertificate" = {
            "secretName" = "traefik-tls"
          }
        }
      }
      "tracing" = {
        "jaeger" = {
          "samplingServerURL"      = "http://localhost:5778/sampling"
          "propagation"            = "jaeger"
          "localAgentHostPort"     = "127.0.0.1:6831"
          "traceContextHeaderName" = "trace-id"
        }
      }
      "ports" = {
        "web" = {
          "expose" = false
        }
        "websecure" = {
          "asDefault" = true
        }
      }
    })
  ]
}

resource "kubectl_manifest" "traefik_service_monitor" {
  depends_on = [helm_release.traefik, helm_release.prometheus]

  yaml_body = yamlencode({
    "apiVersion" = "monitoring.coreos.com/v1"
    "kind"       = "ServiceMonitor"
    "metadata"   = {
      "name"      = "traefik"
      "namespace" = "traefik"
    }
    "spec" = {
      "jobLabel" = "traefik"
      "selector" = {
        "matchLabels" = {
          "app.kubernetes.io/instance" = "traefik-traefik"
          "app.kubernetes.io/name"     = "traefik"
        }
        "namespaceSelector" = {
          "matchNames" = ["traefik"]
        }
      }
      "endpoints" = [
        {
          "port" = "traefik"
          "path" = "/metrics"
        }
      ]
    }
  })
}
