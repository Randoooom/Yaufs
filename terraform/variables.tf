variable "cluster" {
  type    = string
  default = "minikube"
}

variable "host" {
  type = string
  default = "dev.localhost"
}

variable "apisix_namespace" {
  type = string
  default = "apisix"
}

variable "vault_namespace" {
  type = string
  default = "vault"
}

variable "consul_namespace" {
  type = string
  default = "consul"
}

variable "prometheus_namespace" {
  type = string
  default = "prometheus"
}
