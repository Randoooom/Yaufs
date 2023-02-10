#!/bin/bash

kubectl exec -n vault vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-0 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n vault vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-1 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n vault vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[0] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[1] output/vault.json | tr -d '"')"
kubectl exec -n vault vault-2 -- vault operator unseal "$(jq .unseal_keys_b64[2] output/vault.json | tr -d '"')"

kubectl exec -n vault vault-0 -- vault login "$(jq .root_token output/vault.json | tr -d '"')"
