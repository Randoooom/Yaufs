variable "zitadel_admin_password" {
  type = string
}

variable "zitadel_org" {
  type = string
}

variable "host" {
  type = string
}

terraform {
  required_providers {
    zitadel = {
      source  = "zitadel/zitadel"
      version = "1.0.0-alpha.11"
    }
  }
}
