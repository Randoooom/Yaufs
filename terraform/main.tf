terraform {
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.16.1"
    }

    helm = {
      source  = "hashicorp/helm"
      version = "2.8.0"
    }

    kubectl = {
      source  = "gavinbunney/kubectl"
      version = ">= 1.7.0"
    }

    zitadel = {
      source  = "zitadel/zitadel"
      version = "1.0.0-alpha.11"
    }
  }
}
