// META: global=window,dedicatedworker,sharedworker

// Given `{ digest: "...", body: "...", cors: true, type: "..." }`:
function resourceURL(data) {
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

// A canonically validly signed response, generated using the steps at
// https://wicg.github.io/signature-based-sri/#examples, relying on the test
// key from https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key:
//
// ```
// NOTE: '\' line wrapping per RFC 8792
//
// HTTP/1.1 200 OK
// Date: Tue, 20 Apr 2021 02:07:56 GMT
// Content-Type: application/json
// Identity-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
// Content-Length: 18
// Signature-Input: signature=("identity-digest";sf);alg="ed25519"; \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs="; \
//                  tag="sri"
// Signature: signature=:H7AqWWgo1DJ7VdyF9DKotG/4hvatKDfRTq2mpuY/hvJupSn+EYzus \
//            5p24qPK7DtVQcxJFhzSYDj4RBq9grZTAQ==:
//
// {"hello": "world"}
// ```

// Test key from https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key.
const kValidKey   = "JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";

// A key with the right length that cannot be used to verify the HTTP response
// above.
const kInvalidKey = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

// Metadata from the response above:
const kRequestWithValidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:H7AqWWgo1DJ7VdyF9DKotG/4hvatKDfRTq2mpuY/hvJupSn+EYzus5p24qPK7DtVQcxJFhzSYDj4RBq9grZTAQ==:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
};

// Metadata from the response above, but with an incorrect signature:
const kRequestWithInvalidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
};

const EXPECT_BLOCKED = "block";
const EXPECT_LOADED = "loaded";

function generate_test(request_data, integrity, expectation, description) {
  promise_test(test => {
    const url = resourceURL(request_data);
    let options = {};
    if (integrity != "") {
      options.integrity = integrity;
    }

    let fetcher = fetch(url, options);
    if (expectation == EXPECT_LOADED) {
      return fetcher.then(r => {
        assert_equals(r.status, 200, "Response status is 200.");
      });
    } else {
      return promise_rejects_js(test, TypeError, fetcher);
    }
  }, description);
}

generate_test({}, "", EXPECT_LOADED,
              "No signature, no integrity check: loads.");

generate_test({}, `ed25519-!!!`, EXPECT_LOADED,
              "No signature, malformed integrity check: loads.");

generate_test({}, `ed25519-${kValidKey}`, EXPECT_BLOCKED,
              "No signature, valid integrity check: blocked.");

// Valid signatures depend upon integrity checks.
generate_test(kRequestWithValidSignature, "", EXPECT_LOADED,
              "Valid signature, no integrity check: loads.");

generate_test(kRequestWithValidSignature, "ed25519-???", EXPECT_LOADED,
              "Valid signature, malformed integrity check: loads.");

generate_test(kRequestWithValidSignature, `ed25519-${kValidKey}`, EXPECT_LOADED,
              "Valid signature, matching integrity check: loads.");

generate_test(kRequestWithValidSignature, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
              "Valid signature, mismatched integrity check: blocked.");

generate_test(kRequestWithValidSignature,
              `ed25519-${kValidKey} ed25519-${kInvalidKey}`, EXPECT_LOADED,
              "Valid signature, one valid integrity check: loads.");

// Invalid signatures are all blocked.
generate_test(kRequestWithInvalidSignature, "", EXPECT_BLOCKED,
              "Invalid signature, no integrity check: blocked.");

generate_test(kRequestWithInvalidSignature, "ed25519-???", EXPECT_BLOCKED,
              "Invalid signature, malformed integrity check: blocked.");

generate_test(kRequestWithInvalidSignature, `ed25519-${kValidKey}`, EXPECT_BLOCKED,
              "Invalid signature, matching integrity check: blocked.");

generate_test(kRequestWithInvalidSignature, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
              "Invalid signature, mismatched integrity check: blocked.");

generate_test(kRequestWithInvalidSignature,
              `ed25519-${kValidKey} ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
              "Invalid signature, one valid integrity check: blocked.");
