resource "kubernetes_namespace" "jaeger" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "jaeger"
    annotations = {
      "linkerd.io/inject"                        = "enabled"
      "config.linkerd.io/default-inbound-policy" = "cluster-unauthenticated"
    }
  }
}

resource "helm_release" "jaeger_operator" {
  name       = "jaeger"
  namespace  = "jaeger"
  depends_on = [helm_release.cert-manager, kubernetes_namespace.jaeger]

  repository = "https://jaegertracing.github.io/helm-charts"
  chart      = "jaeger-operator"

  values = [
    yamlencode({
      "rbac" = {
        "clusterRole" = true
      }
      "jaeger" = {
        "create"    = true
        "namespace" = "jaeger"
        "spec"      = {
          annotations = {
            "linkerd.io/inject" = "enabled"
          }
          "strategy"  = "production",
          "collector" = {
            "maxReplicas" = 1
          }
          "ingress" = {
            "enabled" = false
          }
        }
      }
    })
  ]
}
