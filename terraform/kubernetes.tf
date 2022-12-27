provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.cluster
}

resource "kubernetes_namespace" "yaufs" {
  metadata {
    name = "yaufs"
  }
}

resource "kubernetes_namespace" "vault" {
  metadata {
    name = "vault"
  }
}

resource "kubernetes_namespace" "consul" {
  metadata {
    name = "consul"
  }
}

resource "kubernetes_namespace" "apisix" {
  metadata {
    name = "apisix"
  }
}
