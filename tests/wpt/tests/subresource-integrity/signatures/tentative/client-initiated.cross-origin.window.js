// META: script=/common/get-host-info.sub.js
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
// Unencoded-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
// Content-Length: 18
// Signature-Input: signature=("unencoded-digest";sf); \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs="; \
//                  tag="sri"
// Signature: signature=:TUznBT2ikFq6VrtoZeC5znRtZugu1U8OHJWoBkOLDTJA2FglSR34Q \
//                       Y9j+BwN79PT4H0p8aIosnv4rXSKfIZVDA==:
//
// {"hello": "world"}
// ```


// Unsigned responses are blocked when integrity is asserted:
generate_fetch_test({},
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                    },
                    EXPECT_BLOCKED,
                    "No signature, valid integrity check, w/o cors: blocked.");
generate_fetch_test({ cors: true },
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: 'cors',
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                    },
                    EXPECT_BLOCKED,
                    "No signature, valid integrity check, w/ cors: blocked.");

// Valid signatures depend upon integrity checks and CORS status.
//
// Valid signature, no cors: all blocked.
const kRequestWithValidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:gHim9e5Pk2H7c9BStOmxSmkyc8+ioZgoxynu3d4INAT4dwfj5LhvaV9DFnEQ9p7C0hzW4o4Qpkm5aApd6WLLCw==:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`
};
generate_fetch_test(kRequestWithValidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                    },
                    EXPECT_BLOCKED,
                    "Valid signature, matching integrity check, no cors: blocked.");

generate_fetch_test(kRequestWithValidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kInvalidKey}`,
                    },
                    EXPECT_BLOCKED,
                    "Valid signature, mismatched integrity check, no cors: blocked.");

generate_fetch_test(kRequestWithValidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity:`ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`
                    },
                    EXPECT_BLOCKED,
                    "Valid signature, one valid integrity check, no cors: loads.");

// Valid signature, CORS: depends on integrity check.
const kRequestWithValidSignatureAndCORS = {
  body: kRequestWithValidSignature['body'],
  digest: kRequestWithValidSignature['digest'],
  signature: kRequestWithValidSignature['signature'],
  signatureInput: kRequestWithValidSignature['signatureInput'],
  cors: true,
};
generate_fetch_test(kRequestWithValidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                    },
                    EXPECT_LOADED,
                    "Valid signature, matching integrity check, cors: loads.");

generate_fetch_test(kRequestWithValidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                      integrity: `ed25519-${kInvalidKey}`,
                    },
                    EXPECT_BLOCKED,
                    "Valid signature, mismatched integrity check, cors: blocked.");

generate_fetch_test(kRequestWithValidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                      integrity:`ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`
                    },
                    EXPECT_LOADED,
                    "Valid signature, one valid integrity check, cors: loads.");


// Invalid signatures, cors or no cors, are all blocked.
const kRequestWithInvalidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`
};
generate_fetch_test(kRequestWithInvalidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, matching integrity check, no cors: blocked.");

generate_fetch_test(kRequestWithInvalidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kInvalidKey}`,
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, mismatched integrity check, no cors: blocked.");

generate_fetch_test(kRequestWithInvalidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity:`ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, one valid integrity check, no cors: loads.");

const kRequestWithInvalidSignatureAndCORS = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
  cors: true
};
generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kValidKeys['rfc']}`,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, matching integrity check, cors: blocked.");

generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity: `ed25519-${kInvalidKey}`,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, mismatched integrity check, cors: blocked.");

generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      integrity:`ed25519-${kValidKeys['rfc']} ed25519-${kInvalidKey}`,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, one valid integrity check, cors: loads.");
