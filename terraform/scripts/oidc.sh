echo "Enabling userpass authentication"
kubectl exec -n vault vault-0 -- vault auth enable userpass

echo "Creating initial vault OIDC admin"
password=$(openssl rand -base64 28 | tr -d "=+/" | cut -c1-24)
echo "username: admin"
echo "password: $password"
kubectl exec -n vault vault-0 -- vault write auth/userpass/users/admin \
  password="$(openssl rand -base64 28 | tr -d "=+/" | cut -c1-24)" \
  token_ttl="1h"
unset password

echo "Adding entity"
kubectl exec -n vault vault-0 -- vault write identity/entity \
  name="admin" \
  disabled=false
ENTITY_ID=$(kubectl exec -n vault vault-0 -- vault read -field=id identity/entity/name/admin)
echo "Creating oic groups"
kubectl exec -n vault vault-0 -- vault write identity/group \
  name="engineering" \
  member_entity_ids="$ENTITY_ID"

GROUP_ID=$(kubectl exec -n vault vault-0 -- vault read -field=id identity/group/name/engineering)
USERPASS_ACCESSOR=$(kubectl exec -n vault vault-0 -- vault auth list -detailed -format json | jq -r '.["userpass/"].accessor')
kubectl exec -n vault vault-0 -- vault write identity/entity-alias \
  name="admin" \
  canonical_id="$ENTITY_ID" \
  mount_accessor="$USERPASS_ACCESSOR"

echo "Creating client"
kubectl exec -n vault vault-0 -- vault write identity/oidc/key/key \
  allowed_client_ids="*" \
  verification_ttl="2h" \
  rotation_period="1h" \
  algorithm="RS256"
kubectl exec -n vault vault-0 -- vault write identity/oidc/client/apisix \
  redirect_uris="https://apisix-gateway.apisix.svc.cluster.local:443/oidc/callback" \
  key="key" \
  id_token_ttl="30m" \
  access_token_ttl="1h"
CLIENT_ID=$(kubectl exec -n vault vault-0 -- vault read -field=client_id identity/oidc/client/apisix)
CLIENT_SECRET=$(kubectl exec -n vault vault-0 -- vault read -field=client_secret identity/oidc/client/apisix)
echo "Apisix Client: $CLIENT_ID"
echo "Apisix Secret: $CLIENT_SECRET"

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
kubectl exec -n vault vault-0 -- vault write identity/oidc/scope/user \
  description="The user scope provides claims using Vault identity entity metadata" \
  template="$(echo ${USER_SCOPE_TEMPLATE} | base64 -)"

kubectl exec -n vault vault-0 -- vault write identity/oidc/scope/groups \
  description="The groups scope provides the groups claim using Vault group membership" \
  template="$(echo ${GROUPS_SCOPE_TEMPLATE} | base64 -)"

echo "Creating provider"
kubectl exec -n vault vault-0 -- vault write identity/oidc/provider/vault \
  allowed_client_ids="${CLIENT_ID}" \
  scopes_supported="groups,user"
ISSUER=$(kubectl exec -n vault vault-0 -- vault read identity/oidc/provider/vault -format=json | jq .data.issuer | tr -d '"')
echo "Issuer: $ISSUER"

echo "Fetching provider key"
PUBLIC_KEY=$(
  curl --request GET https://vault.dev.localhost/v1/identity/oidc/provider/vault/.well-known/keys -k | jq .keys[0].n
)
sed -i 's/replace_public_key/'"$PUBLIC_KEY"'/g' config/apisix/oidc-admin.yaml
echo "Saved key into configuration"
kubectl apply -f config/apisix/oidc-admin.yaml