resource "helm_release" "consul" {
  name      = "consul"
  namespace = "consul"

  repository = "https://helm.releases.hashicorp.com"
  chart      = "consul"
}
