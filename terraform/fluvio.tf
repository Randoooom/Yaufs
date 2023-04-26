resource "kubernetes_namespace" "fluvio" {
  depends_on = [helm_release.linkerd]

  metadata {
    name        = "fluvio"
    annotations = {
      "linkerd.io/inject"                        = "enabled"
      "config.linkerd.io/default-inbound-policy" = "all-unauthenticated"
    }
  }
}

resource "kubectl_manifest" "fluvio" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.fluvio]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "fluvio-server"
      "namespace" = kubernetes_namespace.fluvio.metadata[0].name
    }
    "spec" = {
      "commonName" = "fluvio.local"
      "dnsNames" = ["fluvio.local", "*.fluvio.svc.cluster.local", "*.fluvio.local"]
      "issuerRef"  = {
        "name" = "vault-issuer"
        "kind" = "ClusterIssuer"
      }
      "secretName" = "tls-fluvio-server"
    }
  })
}

resource "kubectl_manifest" "fluvio_client" {
  depends_on = [helm_release.cert-manager, kubectl_manifest.issuer, kubernetes_namespace.fluvio]
  yaml_body  = yamlencode({
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata"   = {
      "name"      = "fluvio-client"
      "namespace" = kubernetes_namespace.fluvio.metadata[0].name
    }
    "spec" = {
      "commonName" = "fluvio.local",
      "issuerRef"  = {
        "name" = "vault-issuer"
        "kind" = "ClusterIssuer"
      }
      "secretName" = "tls-fluvio-client"
    }
  })
}

resource "time_sleep" "wait_for_fluvio_tls" {
  depends_on      = [kubectl_manifest.fluvio_client]
  create_duration = "5s"
}

data "kubernetes_secret" "fluvio_server" {
  depends_on = [kubectl_manifest.fluvio, time_sleep.wait_for_fluvio_tls]
  metadata {
    name      = "tls-fluvio-server"
    namespace = kubernetes_namespace.fluvio.metadata[0].name
  }
}

data "kubernetes_secret" "fluvio" {
  depends_on = [kubectl_manifest.fluvio, time_sleep.wait_for_fluvio_tls]
  metadata {
    name      = "tls-fluvio-client"
    namespace = kubernetes_namespace.fluvio.metadata[0].name
  }
}

resource "local_sensitive_file" "fluvio_client" {
  depends_on = [data.kubernetes_secret.fluvio]
  for_each   = {
    for file in ["tls.crt", "tls.key"] : file => file
  }

  filename = "${path.module}/output/fluvio.client.${each.key}"
  content  = data.kubernetes_secret.fluvio.data[each.key]
}

resource "local_sensitive_file" "fluvio_server" {
  depends_on = [data.kubernetes_secret.fluvio]
  for_each   = {
    for file in ["tls.crt", "tls.key"] : file => file
  }

  filename = "${path.module}/output/fluvio.server.${each.key}"
  content  = data.kubernetes_secret.fluvio_server.data[each.key]
}

resource "null_resource" "fluvio_setup" {
  depends_on = [local_sensitive_file.fluvio_client, local_sensitive_file.fluvio_server]

  provisioner "local-exec" {
    command = <<EOT
                    fluvio cluster start --namespace fluvio --tls --domain fluvio.local \
                      --ca-cert ${path.module}/output/ca.crt \
                      --client-cert ${path.module}/output/fluvio.client.tls.crt \
                      --client-key ${path.module}/output/fluvio.client.tls.key \
                      --server-cert ${path.module}/output/fluvio.server.tls.crt \
                      --server-key ${path.module}/output/fluvio.server.tls.key
                EOT
  }
}

resource "kubectl_manifest" "fluvio_event_topic" {
  depends_on = [null_resource.fluvio_setup]
  yaml_body  = yamlencode({
    "apiVersion" = "fluvio.infinyon.com/v2"
    "kind"       = "Topic"
    "metadata"   = {
      name      = "events"
      namespace = kubernetes_namespace.fluvio.metadata[0].name
    }
    "spec" = {
      "replicas" = {
        "computed" = {
          "partitions"        = 1
          "replicationFactor" = 1
        }
      }
    }
  })
}

