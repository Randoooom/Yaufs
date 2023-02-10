#!/bin/bash

mkdir output

openssl genrsa -out output/vault.key 2048
cat >output/vault-csr.conf <<EOF
[req]
default_bits = 2048
prompt = no
encrypt_key = yes
default_md = sha256
distinguished_name = kubelet_serving
req_extensions = v3_req
[ kubelet_serving ]
O = system:nodes
CN = system:node:*.vault.svc.cluster.local
[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment, dataEncipherment
extendedKeyUsage = serverAuth, clientAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = *.vault-internal
DNS.2 = *.vault-internal.vault.svc.cluster.local
DNS.3 = *.vault
DNS.4 = *.vault.svc
DNS.5 = *.vault.svc.cluster.local
IP.1 = 127.0.0.1
EOF

openssl req -new -key output/vault.key -out output/vault.csr -config output/vault-csr.conf

cat >output/csr.yaml <<EOF
apiVersion: certificates.k8s.io/v1
kind: CertificateSigningRequest
metadata:
   name: vault.svc
spec:
   signerName: kubernetes.io/kubelet-serving
   expirationSeconds: 8640000
   request: $(cat output/vault.csr | base64 | tr -d '\n')
   usages:
   - digital signature
   - key encipherment
   - server auth
EOF
kubectl create -f output/csr.yaml
kubectl certificate approve vault.svc
echo "Issued certificate"

kubectl get csr vault.svc -o jsonpath='{.status.certificate}' | openssl base64 -d -A -out output/vault.crt
echo "Retrieving CA"
kubectl config view \
  --raw \
  --minify \
  --flatten \
  -o jsonpath='{.clusters[].cluster.certificate-authority-data}' |
  base64 -d >output/vault.ca

echo "Creating secret"
kubectl create secret generic vault-ha-tls \
  -n vault \
  --from-file=vault.key=output/vault.key \
  --from-file=vault.crt=output/vault.crt \
  --from-file=vault.ca=output/vault.ca

echo "Cleaning up"
rm output/csr.yaml
rm output/vault-csr.conf
rm output/vault.key
rm output/vault.crt
rm output/vault.csr
