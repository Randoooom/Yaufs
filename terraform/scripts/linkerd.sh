#!/bin/bash

kubectl get secret -n linkerd linkerd-identity-issuer -o json |
  jq -r '.data."ca.crt"' |
  base64 --decode |
  tee output/linkerd.ca
