resource "helm_release" "jaeger_operator" {
  name             = "jaeger"
  namespace        = var.jaeger_namespace
  create_namespace = true
  depends_on       = [helm_release.cert-manager]

  repository = "https://jaegertracing.github.io/helm-charts"
  chart      = "jaeger-operator"

  set {
    name  = "rbac.clusterRole"
    value = true
  }
}

resource "kubectl_manifest" "jaeger" {
  depends_on = [helm_release.jaeger_operator]
  yaml_body  = yamlencode({
    "apiVersion" = "jaegertracing.io/v1"
    "kind"       = "Jaeger"
    "metadata"   = {
      "name"      = "jaeger-default"
      "namespace" = var.jaeger_namespace
    }
    "spec" = {
      "strategy"  = "production",
      "collector" = {
        "maxReplicas" = 1
      }
      "ingress" = {
        "enabled" = false
      }
      "annotations" = {
        "linkerd.io/inject" = "enabled"
      }
    }
  })
}
