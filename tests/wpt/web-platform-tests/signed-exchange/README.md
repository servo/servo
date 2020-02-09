This directory contains tests for the
[b3 implementation snapshot](https://tools.ietf.org/html/draft-yasskin-httpbis-origin-signed-exchanges-impl-03) of the
[Signed HTTP Exchanges](https://wicg.github.io/webpackage/draft-yasskin-http-origin-signed-responses.html).

`resources` directory contains pregenerated signed exchanges and
certificate files. To run the tests in this directory,
`resources/127.0.0.1.sxg.pem` must be added as a trusted certificate
in your OS/browser, or your browser must be configured to ignore
errors for that certificate.

Also, these pregenerated signed exchanges and cert files are likely to
be expired, since they have short lifetime (maximum 7 days). If your
browser does not have an option to ignore certificate errors,
regenerate these files by running `generate-test-sxgs.sh` in the
`resource` directory before running the tests.

`generate-test-sxgs.sh` requires command-line tools in the
[webpackage repository](https://github.com/WICG/webpackage).
To install them, run:
```
go get -u github.com/WICG/webpackage/go/signedexchange/cmd/...
```
