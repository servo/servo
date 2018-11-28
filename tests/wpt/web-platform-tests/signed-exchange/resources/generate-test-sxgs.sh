#!/bin/sh

certfile=127.0.0.1.sxg.pem
keyfile=127.0.0.1.sxg.key
inner_url_origin=https://127.0.0.1:8444
# TODO: Stop hard-coding "web-platform.test" in certUrl when generating
# Signed Exchanges on the fly.
cert_url_origin=https://web-platform.test:8444
sxg_content_type='content-type: application/signed-exchange;v=b2'

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

# A valid Signed Exchange.
gen-signedexchange \
  -version 1b2 \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg-location.sxg \
  -miRecordSize 100

# Request method is HEAD.
gen-signedexchange \
  -version 1b2 \
  -method HEAD \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg-head-request.sxg \
  -miRecordSize 100

# validityUrl is different origin from request URL.
gen-signedexchange \
  -version 1b2 \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content failure.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl https://example.com/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg-invalid-validity-url.sxg \
  -miRecordSize 100

# certUrl is 404 and fallback URL is another signed exchange.
gen-signedexchange \
  -version 1b2 \
  -uri $inner_url_origin/signed-exchange/resources/sxg-location.sxg \
  -status 200 \
  -content failure.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/not_found_$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o fallback-to-another-sxg.sxg \
  -miRecordSize 100

# Nested signed exchange.
gen-signedexchange \
  -version 1b2 \
  -uri "$inner_url_origin/signed-exchange/resources/inner-url.html?fallback-from-nested-sxg" \
  -status 200 \
  -content sxg-location.sxg \
  -responseHeader "$sxg_content_type" \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o nested-sxg.sxg \
  -miRecordSize 100

rm -fr $tmpdir
