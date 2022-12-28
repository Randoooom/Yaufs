provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.cluster
}

provider "helm" {
  kubernetes {
    config_path    = "~/.kube/config"
    config_context = var.cluster
  }
}
