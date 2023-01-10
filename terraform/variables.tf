variable "cluster" {
  type    = string
  default = "minikube"
}

variable "host" {
  type    = string
  default = "dev.localhost"
}

variable "apisix_namespace" {
  type    = string
  default = "apisix"
}

variable "vault_namespace" {
  type    = string
  default = "vault"
}

variable "cert_manager_namespace" {
  type    = string
  default = "cert-manager"
}

variable "linkerd_namespace" {
  type    = string
  default = "linkerd"
}

variable "prometheus_namespace" {
  type    = string
  default = "prometheus"
}

variable "jaeger_namespace" {
  type    = string
  default = "jaeger"
}
