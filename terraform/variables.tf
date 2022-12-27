variable "cluster" {
  type    = string
  default = "minikube"
}

variable "zitadel_master_key" {
  type = string
  default = "MasterkeyNeedsToHave32Characters"
}

variable "domain" {
  type = string
  default = "dev.localhost"
}
