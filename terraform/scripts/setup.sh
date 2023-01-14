#!/bin/bash

HOST=$1
VAULT_NAMESPACE=$2
LINKERD_NAMESPACE=$3

# shellcheck disable=SC2143
while [ "$(kubectl get pods -n vault | grep -c Running)" -ne 4 ]; do
  sleep 1
done

mkdir output
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator init -format json | tee output/vault.json

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

# shellcheck disable=SC2016
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- sh -c 'vault operator raft join \
  -address=https://vault-1.vault-internal:8200 \
  -leader-ca-cert="$(cat /vault/userconfig/vault-ha-tls/vault.ca)" \
  -leader-client-cert="$(cat /vault/userconfig/vault-ha-tls/vault.crt)" \
  -leader-client-key="$(cat /vault/userconfig/vault-ha-tls/vault.key)" \
  https://vault-0.vault-internal:8200'
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

# shellcheck disable=SC2016
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- sh -c 'vault operator raft join \
  -address=https://vault-2.vault-internal:8200 \
  -leader-ca-cert="$(cat /vault/userconfig/vault-ha-tls/vault.ca)" \
  -leader-client-cert="$(cat /vault/userconfig/vault-ha-tls/vault.crt)" \
  -leader-client-key="$(cat /vault/userconfig/vault-ha-tls/vault.key)" \
  https://vault-0.vault-internal:8200'
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault login "$(jq .root_token output/vault.json | tr -d '"')"

echo "Activating metrics gathering"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write sys/internal/counters/config enabled=enable

# enable pki engine
echo "Enabling pki secrets engine"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets enable pki

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets tune -max-lease-ttl=87600h pki
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write -field=certificate pki/root/generate/internal \
  common_name="$HOST" \
  ttl=87600h # | tee output/vault-root.ca

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/config/urls \
  issuing_certificates="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/ca" \
  crl_distribution_points="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/crl"

echo "Adding vault roles"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/roles/linkerd \
  allowed_domains="identity.linkerd.cluster.local" \
  allow_subdomains=true \
  allow_bare_domains=true \
  allow_localhost=true \
  key_type="ec" \
  max_ttl=8760h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/roles/cluster \
  allowed_domains="$HOST,svc.cluster.local" \
  allow_subdomains=true \
  allow_bare_domains=true \
  max_ttl=72h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki_int/config/urls \
  issuing_certificates="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/ca" \
  crl_distribution_points="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/crl"

echo "Allow Kubernetes authentication"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault auth enable kubernetes
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c '
    vault write auth/kubernetes/config \
    kubernetes_host=https://$KUBERNETES_SERVICE_HOST:$KUBERNETES_SERVICE_PORT'

echo "Enabling kv storage for yaufs"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets enable -path=yaufs -version=1 kv

echo "Applying basic policies"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write prometheus-metrics - << EOF
path "/sys/metrics" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write pki-cluster - <<EOF
path "pki*"                        { capabilities = ["read", "list"] }
path "pki/sign/cluster"    { capabilities = ["create", "update"] }
path "pki/issue/cluster"   { capabilities = ["create"] }
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write pki-linkerd - <<EOF
path "pki/root/sign-intermediate"   { capabilities = ["create", "update"] }
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write template-service - <<EOF
path "yaufs/template/*"   { capabilities = ["create", "update", "read"] }
EOF'

echo "Creating authentication roles"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/vault-issuer \
  bound_service_account_names="vault-issuer" \
  bound_service_account_namespaces="cert-manager" \
  policies="pki-cluster" \
  ttl=1h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/linkerd-issuer \
  bound_service_account_names=linkerd-issuer \
  bound_service_account_namespaces="$LINKERD_NAMESPACE" \
  policies="pki-linkerd" \
  ttl=1h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/template-service \
  bound_service_account_names=template-service \
  bound_service_account_namespaces=template-service \
  policies="template-service" \
  ttl=1h

PROMETHEUS_VAULT_TOKEN=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault token create \
  -field=token \
  -policy "prometheus-metrics")
kubectl create namespace prometheus
kubectl create secret generic prometheus-vault-token -n prometheus --from-literal=token="$PROMETHEUS_VAULT_TOKEN"

echo "Executing scripts/oidc.sh"
./scripts/oidc.sh "$HOST" "$VAULT_NAMESPACE" "$LINKERD_NAMESPACE"
echo "Vault setup completed"

echo "Initiating kv for yaufs-services"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write yaufs/template/surrealdb \
  username="$(openssl rand -base64 16)" \
  password="$(openssl rand -base64 32)"

echo "Setup finished"
echo "Vault credentials saved to vault.json"
