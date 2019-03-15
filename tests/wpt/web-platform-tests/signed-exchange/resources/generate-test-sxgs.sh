#!/bin/sh
sxg_version=1b3
certfile=127.0.0.1.sxg.pem
keyfile=127.0.0.1.sxg.key
inner_url_origin=https://127.0.0.1:8444
# TODO: Stop hard-coding "web-platform.test" when generating Signed Exchanges on
# the fly.
wpt_test_origin=https://web-platform.test:8444
wpt_test_remote_origin=https://www1.web-platform.test:8444
cert_url_origin=$wpt_test_origin
sxg_content_type='content-type: application/signed-exchange;v=b3'

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
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-location.sxg \
  -miRecordSize 100

# A signed exchange of unsupported version.
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
  -o sxg-version1b2.sxg \
  -miRecordSize 100

# A valid Signed Exchange for testing referrer which logical origin is the wpt
# test origin.
gen-signedexchange \
  -version $sxg_version \
  -uri $wpt_test_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $wpt_test_origin/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-referrer-same-origin.sxg \
  -miRecordSize 100

# A valid Signed Exchange for testing referrer which logical origin is the wpt
# test remote origin.
gen-signedexchange \
  -version $sxg_version \
  -uri $wpt_test_remote_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $wpt_test_remote_origin/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-referrer-remote-origin.sxg \
  -miRecordSize 100

# A invalid Signed Exchange for testing referrer which logical origin is the wpt
# test origin. Response has Cache-Control: no-store header.
gen-signedexchange \
  -version $sxg_version \
  -uri $wpt_test_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -responseHeader "Cache-Control: no-store" \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $wpt_test_origin/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/invalid-sxg-referrer-same-origin.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# A invalid Signed Exchange for testing referrer which logical origin is the wpt
# test remote origin. Response has Cache-Control: no-store header.
gen-signedexchange \
  -version $sxg_version \
  -uri $wpt_test_remote_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -responseHeader "Cache-Control: no-store" \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $wpt_test_remote_origin/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/invalid-sxg-referrer-remote-origin.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# For check-cert-request.tentative.html
gen-signedexchange \
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/check-cert-request.py \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/check-cert-request.sxg \
  -miRecordSize 100

# validityUrl is different origin from request URL.
gen-signedexchange \
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -content failure.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl https://example.com/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-invalid-validity-url.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# certUrl is 404 and fallback URL is another signed exchange.
gen-signedexchange \
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/sxg/sxg-location.sxg \
  -status 200 \
  -content failure.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/not_found_$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/fallback-to-another-sxg.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Nested signed exchange.
gen-signedexchange \
  -version $sxg_version \
  -uri "$inner_url_origin/signed-exchange/resources/inner-url.html?fallback-from-nested-sxg" \
  -status 200 \
  -content sxg/sxg-location.sxg \
  -responseHeader "$sxg_content_type" \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/nested-sxg.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Fallback URL has non-ASCII UTF-8 characters.
gen-signedexchange \
  -version $sxg_version \
  -ignoreErrors \
  -uri "$inner_url_origin/signed-exchange/resources/🌐📦.html" \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-utf8-inner-url.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Fallback URL has invalid UTF-8 sequence.
gen-signedexchange \
  -version $sxg_version \
  -ignoreErrors \
  -uri "$inner_url_origin/signed-exchange/resources/$(echo -e '\xce\xce\xa9').html" \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-invalid-utf8-inner-url.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Fallback URL has UTF-8 BOM.
gen-signedexchange \
  -version $sxg_version \
  -ignoreErrors \
  -uri "$(echo -e '\xef\xbb\xbf')$inner_url_origin/signed-exchange/resources/inner-url.html" \
  -status 200 \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-inner-url-bom.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Response has Cache-Control: no-store header.
gen-signedexchange \
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -responseHeader "Cache-Control: no-store" \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-noncacheable.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Response has a strict-transport-security header.
gen-signedexchange \
  -version $sxg_version \
  -uri $inner_url_origin/signed-exchange/resources/inner-url.html \
  -status 200 \
  -responseHeader "Strict-Transport-Security: max-age=31536000" \
  -content sxg-location.html \
  -certificate $certfile \
  -certUrl $cert_url_origin/signed-exchange/resources/$certfile.cbor \
  -validityUrl $inner_url_origin/signed-exchange/resources/resource.validity.msg \
  -privateKey $keyfile \
  -date 2018-04-01T00:00:00Z \
  -expire 168h \
  -o sxg/sxg-hsts.sxg \
  -miRecordSize 100 \
  -ignoreErrors true

# Signed Exchange with payload integrity error.
echo 'garbage' | cat sxg/sxg-location.sxg - >sxg/sxg-merkle-integrity-error.sxg

rm -fr $tmpdir
