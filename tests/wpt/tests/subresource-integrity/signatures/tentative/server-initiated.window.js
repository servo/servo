// META: script=/common/get-host-info.sub.js
// META: script=helper.js

// The following tests verify server-initiated integrity checks which validate
// signatures even in the absence of integrity metadata asserted by the client.

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

// Valid metadata from the response above:
const kRequestWithValidSignature = {
  body: `{"hello": "world"}`,
  digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
  signature: `signature=:gHim9e5Pk2H7c9BStOmxSmkyc8+ioZgoxynu3d4INAT4dwfj5LhvaV9DFnEQ9p7C0hzW4o4Qpkm5aApd6WLLCw==:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`
};
generate_fetch_test(kRequestWithValidSignature,
                    {},
                    EXPECT_LOADED,
                    "Valid signature, same-origin: loads.");
generate_fetch_test(kRequestWithValidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Valid signature, cross-origin w/o cors, cors: blocked (because of CORS).");

// Valid metadata for a response sending CORS headers:
const kRequestWithValidSignatureAndCORS = {
  body: kRequestWithValidSignature['body'],
  digest: kRequestWithValidSignature['digest'],
  signature: kRequestWithValidSignature['signature'],
  signatureInput: kRequestWithValidSignature['signatureInput'],
  cors: true,
};
generate_fetch_test(kRequestWithValidSignatureAndCORS,
                    {},
                    EXPECT_LOADED,
                    "Valid signature, same-origin w/ cors: loads.");
generate_fetch_test(kRequestWithValidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                    },
                    EXPECT_LOADED,
                    "Valid signature, cross-origin w/cors, mode: cors: loads.");

// Incorrect signature, no cors:
const kRequestWithInvalidSignature = {
  body: kRequestWithValidSignature['body'],
  digest: kRequestWithValidSignature['digest'],
  signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
  signatureInput: kRequestWithValidSignature['signatureInput'],
};
generate_fetch_test(kRequestWithInvalidSignature,
                    {},
                    EXPECT_BLOCKED,
                    "Invalid signature, same-origin: blocked.");
generate_fetch_test(kRequestWithInvalidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "no-cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, cross-origin w/o cors, mode: no-cors: blocked.");
generate_fetch_test(kRequestWithInvalidSignature,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, cross-origin w/o cors, cors: blocked.");

// Incorrect signature, cors:
const kRequestWithInvalidSignatureAndCORS = {
  body: kRequestWithValidSignature['body'],
  digest: kRequestWithValidSignature['digest'],
  signature: kRequestWithInvalidSignature['signature'],
  signatureInput: kRequestWithValidSignature['signatureInput'],
  cors: true,
};
generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {},
                    EXPECT_BLOCKED,
                    "Invalid signature, same-origin w/ cors: blocked.");
generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "no-cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, cross-origin w/ cors, mode: no-cors: blocked.");
generate_fetch_test(kRequestWithInvalidSignatureAndCORS,
                    {
                      origin: get_host_info().REMOTE_ORIGIN,
                      mode: "cors",
                    },
                    EXPECT_BLOCKED,
                    "Invalid signature, cross-origin w/ cors, mode: cors: blocked.");
