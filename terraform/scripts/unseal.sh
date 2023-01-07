#!/bin/bash

VAULT_NAMESPACE=$1

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n "$VAULT_NAMESPACE" vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n "$VAULT_NAMESPACE" vault-0 -- vault login "$(jq .root_token output/vault.json | tr -d '"')"
