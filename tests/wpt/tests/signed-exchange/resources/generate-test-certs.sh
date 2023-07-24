#!/bin/sh

# Creates a self-signed certificate to use for signing exchanges.
# TODO: Integrate into tools/wptserve/wptserve/sslutils/openssl.py

set -e

openssl ecparam -out 127.0.0.1.sxg.key -name prime256v1 -genkey

openssl req -new -sha256 \
  -key 127.0.0.1.sxg.key \
  -out 127.0.0.1.sxg.csr \
  -subj '/CN=127.0.0.1/O=Test/C=US'

openssl x509 -req -days 3650 \
  -in 127.0.0.1.sxg.csr \
  -extfile 127.0.0.1.sxg.ext \
  -signkey 127.0.0.1.sxg.key \
  -out 127.0.0.1.sxg.pem
