resource "kubernetes_namespace" "fluvio" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "fluvio"
    annotations = {
      "linkerd.io/inject" = "enabled"
    }
  }
}

resource "kubectl_manifest" "fluvio" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.fluvio]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "fluvio"
      "namespace" = kubernetes_namespace.fluvio.metadata[0].name
    }
    "spec" = {
      "commonName" = "fluvio.fluvio.svc.cluster.local"
      "dnsNames"   = ["fluvio.fluvio.svc.cluster.local"]
      "issuerRef"  = {
        "name" = "vault-issuer"
        "kind" = "ClusterIssuer"
      }
      "secretName" = "fluvio-tls"
    }
  })
}

resource "time_sleep" "wait_for_fluvio_tls" {
  depends_on      = [kubectl_manifest.fluvio]
  create_duration = "5s"
}

data "kubernetes_secret" "fluvio" {
  depends_on = [kubectl_manifest.fluvio, time_sleep.wait_for_fluvio_tls]
  metadata {
    name      = "fluvio-tls"
    namespace = kubernetes_namespace.fluvio.metadata[0].name
  }
}

resource "kubernetes_secret" "fluvio_ca" {
  depends_on = [kubernetes_namespace.fluvio]
  metadata {
    name      = "fluvio-ca"
    namespace = kubernetes_namespace.fluvio.metadata[0].name
  }

  data = {
    "ca.crt" = data.kubernetes_secret.fluvio.data["ca.crt"]
  }
}

resource "helm_release" "fluvio_crd" {
  name       = "fluvio-sys"
  namespace  = kubernetes_namespace.fluvio.metadata[0].name
  depends_on = [kubernetes_namespace.fluvio]

  chart = "${path.module}/config/fluvio-chart-sys.tgz"
}

resource "helm_release" "fluvio" {
  name       = "fluvio-app"
  namespace  = kubernetes_namespace.fluvio.metadata[0].name
  depends_on = [helm_release.fluvio_crd, kubectl_manifest.fluvio]

  chart = "${path.module}/config/fluvio-chart-app.tgz"

  values = [
    yamlencode({
      "tls"  = true
      "cert" = {
        "domain" = "fluvio.local"
        "caCert" = "fluvio-tls"
        "tls"    = "fluvio-tls"
      }
    })
  ]
}

resource "kubectl_manifest" "fluvio_group" {
  depends_on = [helm_release.fluvio]

  yaml_body = yamlencode({
    "apiVersion" = "fluvio.infinyon.com/v1"
    "kind"       = "SpuGroup"
    "metadata"   = {
      "name"      = "default"
      "namespace" = kubernetes_namespace.fluvio.metadata[0].name
    }
    "spec" = {
      "replicas" = 1
      "minId"    = 10
      "template" = {
        "spec" = {
          "publicEndpoint" = {
            "port"       = 9005
            "ingress"    = []
            "encryption" = "SSL"
          }
          "privateEndpoint" = {
            "port"       = 9006
            "host"       = "localhost"
            "encryption" = "SSL"
          }
        }
      }
    }
  })
}
