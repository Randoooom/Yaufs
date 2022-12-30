#!/bin/bash

HOST=$1
VAULT_NAMESPACE=$2

echo "Enabling userpass authentication"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault auth enable userpass

echo "Creating initial vault OIDC admin"
password=$(openssl rand -base64 28 | tr -d "=+/" | cut -c1-24 | tee output/oidc-admin)
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write auth/userpass/users/admin \
  password="$password" \
  token_ttl="1h"
unset password

echo "Adding entity"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/entity \
  name="admin" \
  disabled=false
ENTITY_ID=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault read -field=id identity/entity/name/admin)
echo "Creating oic groups"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/group \
  name="engineering" \
  member_entity_ids="$ENTITY_ID"

USERPASS_ACCESSOR=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault auth list -detailed -format json | jq -r '.["userpass/"].accessor')
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/entity-alias \
  name="admin" \
  canonical_id="$ENTITY_ID" \
  mount_accessor="$USERPASS_ACCESSOR"

echo "Creating client"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/oidc/key/key \
  allowed_client_ids="*" \
  verification_ttl="2h" \
  rotation_period="1h" \
  algorithm="RS256"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/oidc/client/apisix \
  redirect_uris="https://${HOST}/oidc/callback" \
  assignments="allow_all" \
  key="key" \
  id_token_ttl="30m" \
  access_token_ttl="1h"

CLIENT_ID=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault read -field=client_id identity/oidc/client/apisix)
CLIENT_SECRET=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault read -field=client_secret identity/oidc/client/apisix)

echo "Creating templates"
USER_SCOPE_TEMPLATE='{
     "username": {{identity.entity.name}},
     "contact": {
         "email": {{identity.entity.metadata.email}},
         "phone_number": {{identity.entity.metadata.phone_number}}
     }
 }'
GROUPS_SCOPE_TEMPLATE='{
     "groups": {{identity.entity.groups.names}}
 }'
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/oidc/scope/user \
  description="The user scope provides claims using Vault identity entity metadata" \
  template="$(echo ${USER_SCOPE_TEMPLATE} | base64 -)"

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/oidc/scope/groups \
  description="The groups scope provides the groups claim using Vault group membership" \
  template="$(echo ${GROUPS_SCOPE_TEMPLATE} | base64 -)"

echo "Creating provider"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault write identity/oidc/provider/vault \
  allowed_client_ids="${CLIENT_ID}" \
  scopes_supported="groups,user" \
  issuer="https://vault.${HOST}"
ISSUER=$(kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault read identity/oidc/provider/vault -format=json | jq .data.issuer | tr -d '"')
echo "Issuer: $ISSUER"

echo "Fetching provider key"
PUBLIC_KEY=$(curl --request GET "http://vault.${VAULT_NAMESPACE}.svc.cluster.local:8200/v1/identity/oidc/provider/vault/.well-known/keys" -k | jq .keys[0].n)

echo "Writing data to output/output.json for terraform"
jq -n \
  --arg client_id "$CLIENT_ID" \
  --arg client_secret "$CLIENT_SECRET" \
  --arg public_key "$PUBLIC_KEY" \
  '{client_id: $client_id, client_secret: $client_secret, public_key: $public_key}' | tee output/output.json
