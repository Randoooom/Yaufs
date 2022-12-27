resource "helm_release" "vault" {
  name      = "vault"
  namespace = "vault"

  repository = "https://helm.releases.hashicorp.com"
  chart      = "vault"

  set {
    name = "injector.enabled"
    value = true
  }
}
