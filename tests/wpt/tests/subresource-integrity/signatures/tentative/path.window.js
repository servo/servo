// META: script=helper.js

// The following tests validate the behavior of the `@path` derived component.
// They'll all be rooted in the following response, generated using the steps at
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
// Signature-Input: signature=("identity-digest";sf "@path";req);alg="ed25519"; \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";       \
//                  tag="sri"
// Signature: signature=:oVQ+s/OqXLAVdfvgZ3HaPiyzkpNXZSit9l6e1FB/gOOL3t8FOrIRDV \
//                       CkcIEcJjd3MA1mROn39/WQShTmnKmlDg==:
//
//
// {"hello": "world"}
// ```
//
// TODO: When we remove the `tentative` label from the path, we'll need to
// regenerate the expected signatures below, as the signature base will change.

// Metadata from the response above:
const kRequestsWithValidSignature = [
  // `identity-digest` then `@path`, with the following signature base:
  //
  // ```
  // "identity-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@path";req: /subresource-integrity/signatures/tentative/resource.py
  // "@signature-params": ("identity-digest";sf "@path";req);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  // ```
  {
    body: "window.hello = `world`;",
    digest: "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:",
    signature: `signature=:AEW2XbDmmBK71KBle0Dx1JAWAO7B4QdEH2Tw71c9nntjUmx8xF5t8xbsETRHFwULrvJ4STBFtdMVm5a7QIw5Cw==:`,
    signatureInput: `signature=("identity-digest";sf "@path";req);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`
  },
  // `@path` then `identity-digest`, with the following signature base:
  //
  // ```
  // "@path";req: /subresource-integrity/signatures/tentative/resource.py
  // "identity-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@signature-params": ("@path";req "identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  // ```
  {
    body: "window.hello = `world`;",
    digest: "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:",
    signature: `signature=:NEmnhhW1aKxO+ReWQmmSF17i49ZEdtDC4lRI2CJDw2E/rz9j2a8f8kIwVk7W/BIuQ6kejTAQ2FReGmmkREXPDg==:`,
    signatureInput: `signature=("@path";req "identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`
  }
];

// Valid signatures depend upon integrity checks.
//
// We're testing our handling of malformed and multiple keys generally in
// `fetch.any.js` and `script.window.js`. Here we'll just focus on ensuring
// that responses with `@path` components load at all (no integrity check),
// load when integrity checks match, and fail when integrity checks mismatch.
for (const request of kRequestsWithValidSignature) {
    // fetch():
    generate_fetch_test(request, "", EXPECT_LOADED,
                        `Valid signature (${request.signature}), no integrity check: loads.`);

    generate_fetch_test(request, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                        `Valid signature (${request.signature}), matching integrity check: loads.`);

    generate_fetch_test(request, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                        `Valid signature (${request.signature}), mismatched integrity check: blocked.`);

    // <script>:
    generate_script_test(request, "", EXPECT_LOADED,
                        `Valid signature (${request.signature}), no integrity check: loads.`);
    generate_script_test(request, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                        `Valid signature (${request.signature}), matching integrity check: loads.`);
    generate_script_test(request, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                        `Valid signature (${request.signature}), mismatched integrity check: blocked.`);
}
