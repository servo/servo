// META: global=window,dedicatedworker,sharedworker
// META: script=helper.js

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
// Signature: signature=:TUznBT2ikFq6VrtoZeC5znRtZugu1U8OHJWoBkOLDTJA2FglSR34Q \
//                       Y9j+BwN79PT4H0p8aIosnv4rXSKfIZVDA==:
//
// {"hello": "world"}
// ```

// Metadata from the response above:
const kRequestWithValidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:TUznBT2ikFq6VrtoZeC5znRtZugu1U8OHJWoBkOLDTJA2FglSR34QY9j+BwN79PT4H0p8aIosnv4rXSKfIZVDA==:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`
};

// Metadata from the response above, but with an incorrect signature:
const kRequestWithInvalidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`
};

generate_fetch_test({}, "", EXPECT_LOADED,
                    "No signature, no integrity check: loads.");

generate_fetch_test({}, `ed25519-!!!`, EXPECT_LOADED,
                    "No signature, malformed integrity check: loads.");

generate_fetch_test({}, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
                    "No signature, valid integrity check: blocked.");

// Valid signatures depend upon integrity checks.
generate_fetch_test(kRequestWithValidSignature, "", EXPECT_LOADED,
                    "Valid signature, no integrity check: loads.");

generate_fetch_test(kRequestWithValidSignature, "ed25519-???", EXPECT_LOADED,
                    "Valid signature, malformed integrity check: loads.");

generate_fetch_test(kRequestWithValidSignature, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                    "Valid signature, matching integrity check: loads.");

generate_fetch_test(kRequestWithValidSignature, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                    "Valid signature, mismatched integrity check: blocked.");

generate_fetch_test(kRequestWithValidSignature,
                    `ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`, EXPECT_LOADED,
                    "Valid signature, one valid integrity check: loads.");

// Invalid signatures are all blocked.
generate_fetch_test(kRequestWithInvalidSignature, "", EXPECT_BLOCKED,
                    "Invalid signature, no integrity check: blocked.");

generate_fetch_test(kRequestWithInvalidSignature, "ed25519-???", EXPECT_BLOCKED,
                    "Invalid signature, malformed integrity check: blocked.");

generate_fetch_test(kRequestWithInvalidSignature, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
                    "Invalid signature, matching integrity check: blocked.");

generate_fetch_test(kRequestWithInvalidSignature, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                    "Invalid signature, mismatched integrity check: blocked.");

generate_fetch_test(kRequestWithInvalidSignature,
                    `ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                    "Invalid signature, one valid integrity check: blocked.");
