#!/bin/bash

# kubectl exec -n vault vault-0 -- vault operator init
# kubectl exec -n vault vault-0 -- vault operator unseal token
# kubectl exec -n vault vault-0 -- vault operator unseal token
# kubectl exec -n vault vault-0 -- vault operator unseal token
# kubectl exec -n vault vault-0 -- vault login token

read -p "Please enter the base domain for apisix: " domain

# activate the kv-v2 storage type
echo "Activating kv-v2 secrets engine"
kubectl exec -n vault vault-0 -- vault secrets enable -path=consul kv-v2
# generate the gossip key
echo "Generating the gossip key for kv-v2"
kubectl exec -n vault vault-0 -- vault kv put consul/secret/gossip gossip="$(
  openssl rand -base64 32
)"

# enable pki engine
echo "Enabling pki secrets engine"
kubectl exec -n vault vault-0 -- vault secrets enable pki

# sign certificate
echo "Generating consul certificate"
kubectl exec -n vault vault-0 -- vault secrets tune -max-lease-ttl=87600h pki
kubectl exec -n vault vault-0 -- vault write -field=certificate pki/root/generate/internal \
  common_name="dc1.consul" \
  ttl=87600h | tee consul_ca.crt

# setup certificate configuration
echo "Adding vault roles"
kubectl exec -n vault vault-0 -- vault write pki/roles/consul-server \
  allowed_domains="dc1.consul,consul-server,consul-server.consul,consul-server.consul.svc" \
  allow_subdomains=true \
  allow_bare_domains=true \
  allow_localhost=true \
  max_ttl="720h"

kubectl exec -n vault vault-0 -- vault write pki/roles/apisix \
  allowed_domains=$domain \
  allow_subdomains=true \
  max_ttl=72h

kubectl exec -n vault vault-0 -- vault write pki/config/urls \
  issuing_certificates="http://vault.vault.svc.cluster.local:8200/v1/pki/ca" \
  crl_distribution_points="http://vault.vault.svc.cluster.local:8200/v1/pki/crl"

#kubectl exec -n vault vault-0 -- vault write pki/roles/apisix \
#  bound_service_account_names=apisix \
#  bound_service_account_namespaces=apisix \
#  policies="gossip-policy,ca-policy" \
#  allow_subdomains=true \
#  allow_bare_domains=true \
#  allow_localhost=true \
#  allowed_domains=$($domain) \
#  ttl=8765h

kubectl exec -n vault vault-0 -- vault secrets enable -path connect-root pki

echo "Allow Kubernetes authentication"
kubectl exec -n vault vault-0 -- vault auth enable kubernetes
kubectl exec -n vault vault-0 -- sh -c '
    vault write auth/kubernetes/config \
    kubernetes_host=https://$KUBERNETES_SERVICE_HOST:$KUBERNETES_SERVICE_PORT'

echo "Applying basic policies"
kubectl exec -n vault vault-0 -- sh -c 'vault policy write gossip-policy - <<EOF
path "consul/data/secret/gossip" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n vault vault-0 -- sh -c 'vault policy write pki - <<EOF
path "pki*"                        { capabilities = ["read", "list"] }
path "pki/sign/apisix"    { capabilities = ["create", "update"] }
path "pki/issue/apisix"   { capabilities = ["create"] }
EOF'

kubectl exec -n vault vault-0 -- sh -c 'vault policy write consul-server - <<EOF
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

kubectl exec -n vault vault-0 -- sh -c 'vault policy write ca-policy - <<EOF
path "pki/cert/ca" {
  capabilities = ["read"]
}
EOF'

kubectl exec -n vault vault-0 -- sh -c 'vault policy write connect - <<EOF
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
kubectl exec -n vault vault-0 -- vault write auth/kubernetes/role/consul-server \
  bound_service_account_names=consul-server \
  bound_service_account_namespaces=consul \
  policies="gossip-policy,consul-server,connect" \
  ttl=24h

kubectl exec -n vault vault-0 -- vault write auth/kubernetes/role/consul-client \
  bound_service_account_names=consul-client \
  bound_service_account_namespaces=consul \
  policies="gossip-policy,ca-policy" \
  ttl=24h

kubectl exec -n vault vault-0 -- vault write auth/kubernetes/role/consul-ca \
  bound_service_account_names="*" \
  bound_service_account_namespaces=consul \
  policies=ca-policy \
  ttl=1h

kubectl exec -n vault vault-0 -- vault write auth/kubernetes/role/issuer \
  bound_service_account_names=issuer \
  bound_service_account_namespaces=default \
  policies=pki \
  ttl=1h

echo "Executing scripts/oidc.sh"
./scripts/oidc.sh

echo "Vault setup completed"

echo "Creating serviceaccount for cert-manager"
kubectl create serviceaccount -n apisix vault-issuer

echo "Please proceed with adjusting the config"
echo "After that please execute scripts/deploy.sh"
