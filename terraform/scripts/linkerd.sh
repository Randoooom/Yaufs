#!/bin/bash

LINKERD_NAMESPACE=$1

kubectl get secret -n "$LINKERD_NAMESPACE" linkerd-identity-issuer -o json |
  jq -r '.data."ca.crt"' |
  base64 --decode |
  tee output/linkerd.ca
