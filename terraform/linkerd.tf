resource "helm_release" "linkerd_crds" {
  name             = "linkerd-crds"
  namespace        = var.linkerd_namespace
  create_namespace = true

  repository = "https://helm.linkerd.io/stable"
  chart      = "linkerd-crds"
}

resource "kubernetes_service_account" "linkerd_issuer_serviceaccount" {
  depends_on = [helm_release.linkerd_crds, helm_release.cert-manager, null_resource.vault_setup]

  metadata {
    name      = "linkerd-issuer"
    namespace = var.linkerd_namespace
  }
}

resource "kubernetes_secret" "linkerd_issuer_token" {
  depends_on = [kubernetes_service_account.linkerd_issuer_serviceaccount]

  metadata {
    name        = "linkerd-issuer-token"
    namespace   = var.linkerd_namespace
    annotations = {
      "kubernetes.io/service-account.name" = "linkerd-issuer"
    }
  }
  type = "kubernetes.io/service-account-token"
}

resource "kubectl_manifest" "linkerd_issuer" {
  depends_on = [kubernetes_secret.linkerd_issuer_token]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Issuer"
    "metadata"   = {
      "name"      = "linkerd-trust-anchor"
      "namespace" = var.linkerd_namespace
    }
    "spec" = {
      "vault" = {
        "auth" = {
          "kubernetes" = {
            "mountPath" = "/v1/auth/kubernetes"
            "role"      = "linkerd-issuer"
            "secretRef" = {
              "key"  = "token"
              "name" = "linkerd-issuer-token"
            }
          }
        }
        "caBundle" = data.local_file.vault_ca.content_base64
        "path"     = "pki/root/sign-intermediate"
        "server"   = "https://vault.${var.vault_namespace}.svc.cluster.local:8200"
      }
    }
  })
}

resource "time_sleep" "wait_for_linkerd_issuer" {
  depends_on      = [kubectl_manifest.linkerd_issuer]
  create_duration = "10s"
}

resource "kubectl_manifest" "linkerd_intermediate_ca" {
  depends_on = [time_sleep.wait_for_linkerd_issuer]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "linkerd-identity-issuer"
      "namespace" = var.linkerd_namespace
    }
    "spec" = {
      "isCA"       = true
      "commonName" = "identity.linkerd.cluster.local"
      "dnsNames"   = ["identity.linkerd.cluster.local"]
      "privateKey" = {
        "algorithm" = "ECDSA"
      }
      "issuerRef" = {
        "name" = "linkerd-trust-anchor"
        "kind" = "Issuer"
      }
      "secretName" = "linkerd-identity-issuer"
      "usages"     = ["cert sign", "crl sign", "server auth", "client auth"]
    }
  })
}

resource "null_resource" "read_linkerd_ca" {
  depends_on = [kubectl_manifest.linkerd_intermediate_ca]
  provisioner "local-exec" {
    command = "sleep 10; chmod +x scripts/linkerd.sh; /bin/bash scripts/linkerd.sh ${var.linkerd_namespace}"
  }
}

data "local_file" "linkerd_ca" {
  depends_on = [null_resource.read_linkerd_ca]
  filename   = "${path.module}/output/linkerd.ca"
}

resource "helm_release" "linkerd" {
  name       = "linkerd"
  namespace  = var.linkerd_namespace
  depends_on = [data.local_file.linkerd_ca, helm_release.linkerd_crds]

  repository = "https://helm.linkerd.io/stable"
  chart      = "linkerd-control-plane"

  values = [
    yamlencode({
      "identityTrustAnchorsPEM" = data.local_file.linkerd_ca.content
      "identity"                = {
        "issuer" = {
          "scheme" = "kubernetes.io/tls"
        }
      }
    })
  ]
}

