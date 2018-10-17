#!/bin/sh
certfile=127.0.0.1.sxg.pem
keyfile=127.0.0.1.sxg.key
host=127.0.0.1

set -e

for cmd in gen-signedexchange gen-certurl; do
    if ! command -v $cmd > /dev/null 2>&1; then
        echo "$cmd is not installed. Please run:"
        echo "  go get -u github.com/WICG/webpackage/go/signedexchange/cmd/..."
        exit 1
    fi
done

tmpdir=$(mktemp -d)

echo -n OCSP >$tmpdir/ocsp
gen-certurl -pem $certfile -ocsp $tmpdir/ocsp > $certfile.cbor

# TODO: Stop hard-coding "web-platform.test" in certUrl when generating
# Signed Exchanges on the fly.
gen-signedexchange \
  -version 1b2 \
  -uri https://$host/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl https://web-platform.test:8444/signed-exchange/resources/$certfile.cbor \
  -validityUrl https://$host/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg-location.sxg \
  -miRecordSize 100

rm -fr $tmpdir
