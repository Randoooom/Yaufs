resource "helm_release" "cert-manager" {
  name = "cert-manager"
  namespace = "apisix"
  depends_on = [helm_release.vault]

  repository = "https://charts.jetstack.io"
  chart = "cert-manager"

  set {
    name = "prometheus.enabled"
    value = false
  }

  set {
    name = "installCRDs"
    value = true
  }
}

resource "helm_release" "apisix" {
  name      = "apisix"
  namespace = "apisix"
  depends_on = [helm_release.cert-manager]

  repository = "https://charts.apiseven.com"
  chart      = "apisix"

  set {
    name = "ingress-controller.enabled"
    value = true
  }

  set {
    name = "ingress-controller.config.apisix.serviceNamespace"
    value = "apisix"
  }

  set {
    name = "gateway.type"
    value = "NodePort"
  }

  set {
    name = "gateway.tls.enabled"
    value = "true"
  }

  set {
    name = "discovery.enabled"
    value = true
  }

  set {
    name = "discovery.consul_kv.servers"
    value = yamlencode(["consul.consul.svc.cluster.local:8501"])
  }

  set {
    name = "discovery.consul_kv.prefix"
    value = "upstreams"
  }
}
