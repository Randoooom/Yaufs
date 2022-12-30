#!/bin/bash

HOST=$1
VAULT_NAMESPACE=$2
CONSUL_NAMESPACE=$3

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

# activate the kv-v2 storage type
echo "Activating kv-v2 secrets engine"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets enable -path=consul kv-v2
# generate the gossip key
echo "Generating the gossip key for kv-v2"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault kv put consul/secret/gossip gossip="$(
  openssl rand -base64 32
)"
echo "Generating bootstrapToken"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault kv put consul/secret/bootstrap-token \
  token="$(uuidgen | tr '[:upper:]' '[:lower:]')"

# enable pki engine
echo "Enabling pki secrets engine"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets enable pki

# sign certificate
echo "Generating consul certificate"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets tune -max-lease-ttl=87600h pki
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write -field=certificate pki/root/generate/internal \
  common_name="dc1.consul" \
  ttl=87600h

# setup certificate configuration
echo "Adding vault roles"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/roles/consul-server \
  allowed_domains="dc1.consul,consul-server,consul-server.consul,consul-server.consul.svc" \
  allow_subdomains=true \
  allow_bare_domains=true \
  allow_localhost=true \
  max_ttl="720h"

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/roles/apisix \
  allowed_domains="$HOST" \
  allow_subdomains=true \
  max_ttl=72h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write pki/config/urls \
  issuing_certificates="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/ca" \
  crl_distribution_points="http://vault.$VAULT_NAMESPACE.svc.cluster.local:8200/v1/pki/crl"

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault secrets enable -path connect-root pki

echo "Allow Kubernetes authentication"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault auth enable kubernetes
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c '
    vault write auth/kubernetes/config \
    kubernetes_host=https://$KUBERNETES_SERVICE_HOST:$KUBERNETES_SERVICE_PORT'

echo "Applying basic policies"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write gossip-policy - <<EOF
path "consul/data/secret/gossip" {
  capabilities = ["read"]
}
EOF'
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write bootstrap-token-policy - <<EOF
path "consul/data/secret/bootstrap-token" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write prometheus-metrics - << EOF
path "/sys/metrics" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write pki - <<EOF
path "pki*"                        { capabilities = ["read", "list"] }
path "pki/sign/apisix"    { capabilities = ["create", "update"] }
path "pki/issue/apisix"   { capabilities = ["create"] }
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write consul-server - <<EOF
path "kv/data/consul-server"
{
  capabilities = ["read"]
}
path "pki/issue/consul-server"
{
  capabilities = ["read","update"]
}
path "pki/cert/ca"
{
  capabilities = ["read"]
}
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write ca-policy - <<EOF
path "pki/cert/ca" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- sh -c 'vault policy write connect - <<EOF
path "/sys/mounts/connect-root" {
  capabilities = [ "create", "read", "update", "delete", "list" ]
}

path "/sys/mounts/connect-intermediate-dc1" {
  capabilities = [ "create", "read", "update", "delete", "list" ]
}

path "/sys/mounts/connect-intermediate-dc1/tune" {
  capabilities = [ "update" ]
}

path "/connect-root/*" {
  capabilities = [ "create", "read", "update", "delete", "list" ]
}

path "/connect-intermediate-dc1/*" {
  capabilities = [ "create", "read", "update", "delete", "list" ]
}

path "auth/token/renew-self" {
  capabilities = [ "update" ]
}

path "auth/token/lookup-self" {
  capabilities = [ "read" ]
}
EOF'

echo "Creating authentication roles"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/consul-server \
  bound_service_account_names="consul-server,consul-server-acl-init" \
  bound_service_account_namespaces="$CONSUL_NAMESPACE" \
  policies="gossip-policy,consul-server,connect,bootstrap-token-policy" \
  ttl=24h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/consul-client \
  bound_service_account_names=consul-client \
  bound_service_account_namespaces="$CONSUL_NAMESPACE" \
  policies="gossip-policy,ca-policy" \
  ttl=24h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/consul-ca \
  bound_service_account_names="*" \
  bound_service_account_namespaces="$CONSUL_NAMESPACE" \
  policies=ca-policy \
  ttl=1h

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/kubernetes/role/issuer \
  bound_service_account_names=issuer \
  bound_service_account_namespaces=default \
  policies=pki \
  ttl=1h

PROMETHEUS_VAULT_TOKEN=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault token create \
  -field=token \
  -policy "prometheus-metrics")
kubectl create namespace prometheus
kubectl create secret generic prometheus-vault-token -n prometheus --from-literal=token="$PROMETHEUS_VAULT_TOKEN"

echo "Executing scripts/oidc.sh"
./scripts/oidc.sh "$HOST" "$VAULT_NAMESPACE" "$CONSUL_NAMESPACE"

echo "Vault setup completed"

echo "Creating serviceaccount for cert-manager"
kubectl create serviceaccount -n default issuer

echo "Setup finished"
echo "Vault credentials saved to vault.json"
