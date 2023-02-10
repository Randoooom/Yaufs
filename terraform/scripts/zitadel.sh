#!/bin/bash

kubectl get secret -n zitadel zitadel-admin-sa -o json |
  jq -r '.data."zitadel-admin-sa.json"' |
  base64 --decode |
  tee output/zitadel.json
