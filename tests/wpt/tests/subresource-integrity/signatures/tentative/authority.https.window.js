// META: script=helper.js

// The following tests validate the behavior of the `@authority` derived
// component.
//
// Since the authority is dependent on the runtime environment, we can't vary
// the authority value freely, and these tests must sign the headers live using
// the WebCrypto API. Usage of that API restricts this test to secure contexts.
//
// These tests are all be rooted in the following response, generated using the
// steps at https://wicg.github.io/signature-based-sri/#examples, relying on
// the test key from
// https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key:
//
// ```
// NOTE: '\' line wrapping per RFC 8792
//
// HTTP/1.1 200 OK
// Date: Tue, 20 Apr 2021 02:07:56 GMT
// Content-Type: application/json
// Unencoded-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
// Content-Length: 18
// Signature-Input: signature=("unencoded-digest";sf "@authority"); \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";       \
//                  tag="sri"
// Signature: signature=:oVQ+s/OqXLAVdfvgZ3HaPiyzkpNXZSit9l6e1FB/gOOL3t8FOrIRDV \
//                       CkcIEcJjd3MA1mROn39/WQShTmnKmlDg==:
//
//
// {"hello": "world"}
// ```

const kAuthority = (new URL(window.location.href)).host;

// Metadata from the response above:
const kRequestsWithValidSignature = [
  // `unencoded-digest` then `@authority`.
  {
    body: "window.hello = `world`;",
    digest: "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:",
    signatureInput: `signature=("unencoded-digest";sf "@authority";req);keyid="${kValidKeys['rfc']}";tag="sri"`,
    signatureBase: `"unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
"@authority";req: ${kAuthority}
"@signature-params": ("unencoded-digest";sf "@authority";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
  },

  // `@authority` then `unencoded-digest`.
  {
    body: "window.hello = `world`;",
    digest: "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:",
    signatureInput: `signature=("@authority";req "unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
    signatureBase: `"@authority";req: ${kAuthority}
"unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
"@signature-params": ("@authority";req "unencoded-digest";sf);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
  }
];

// Valid signatures depend upon integrity checks.
//
// We're testing our handling of malformed and multiple keys generally in
// the broader `client-initiated.*` tests. Here we'll just focus on ensuring
// that responses with `@authority` components load at all (no integrity check),
// load when integrity checks match, and fail when integrity checks mismatch.
for (const constRequest of kRequestsWithValidSignature) {
    signSignatureBase(constRequest.signatureBase, kValidKeysJWK['rfc']).then(plainSignature => {
      let request = {
        ...constRequest,
        signature: `signature=:` + plainSignature + `:`,
      };

      // fetch():
      generate_fetch_test(request, {}, EXPECT_LOADED,
                          `Valid signature (${request.signature}), no integrity check: loads.`);
      generate_fetch_test(request, {integrity:`ed25519-${kValidKeys['rfc']}`}, EXPECT_LOADED,
                          `Valid signature (${request.signature}), matching integrity check: loads.`);
      generate_fetch_test(request, {integrity:`ed25519-${kInvalidKey}`}, EXPECT_BLOCKED,
                          `Valid signature (${request.signature}), mismatched integrity check: blocked.`);

      // <script>:
      generate_script_test(request, "", EXPECT_LOADED,
                          `Valid signature (${request.signature}), no integrity check: loads with live signature.`);
      generate_script_test(request, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                          `Valid signature (${request.signature}), matching integrity check: loads with live signature.`);
      generate_script_test(request, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                          `Valid signature (${request.signature}), mismatched integrity check: blocked.`);
    });
}

