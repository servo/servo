// META: script=helper.js

// The following tests validate the behavior of arbitrary header components in
// signatures. They rely on the test key from
// https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key
//
// TODO: Replace the placeholder signatures below with real signatures.

const kBody = "window.hello = `world`;";
const kDigest = "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:";

const kSignatures = {
  // Signature Base:
  // "content-type": application/javascript
  // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@signature-params": ("content-type" "unencoded-digest";sf);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  ct_then_digest: "signature=:a5uYGT79upHlmAkq3PsPHr1xz5AXjqmjVbOr38e8HW94+YqthWLjPeYVRYkYprb0zDqrreptOB4m5d148uWAAQ==:",

  // Signature Base:
  // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "content-type": application/javascript
  // "@signature-params": ("unencoded-digest";sf "content-type");keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  digest_then_ct: "signature=:OuJoPVjT+cucaPpFGs3hyhdOGcEEgCAaLefYWCwEGdsBFunmFAT6hbMTg/vmhLM3raZKKzWqjFkzinWtia8fAA==:",

  // Signature Base:
  // "content-type": application/javascript
  // "x-extra-header": some-value
  // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@signature-params": ("content-type" "x-missing-header" "unencoded-digest";sf);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  with_extra_header: "signature=:9UFhYYWqUSxadNI4vyPSln1deZGswee7DpRg761SaORKykyewqhD6pEPJBXfEaV9wJrqgq4Nq+oJb2Pe25TjDA==:",
};

const kRequestsWithValidSignature = [
  {
    body: kBody,
    digest: kDigest,
    signature: kSignatures.ct_then_digest,
    signatureInput: `signature=("content-type" "unencoded-digest";sf);keyid="${kValidKeys["rfc"]}";tag="sri"`
  },
  {
    body: kBody,
    digest: kDigest,
    signature: kSignatures.digest_then_ct,
    signatureInput: `signature=("unencoded-digest";sf "content-type");keyid="${kValidKeys["rfc"]}";tag="sri"`
  }
];

for (const request of kRequestsWithValidSignature) {
    // fetch():
    generate_fetch_test(request, {}, EXPECT_LOADED,
                        `Valid signature (${request.signatureInput}), no integrity check: loads.`);
    generate_fetch_test(request, {integrity:`ed25519-${kValidKeys["rfc"]}`}, EXPECT_LOADED,
                        `Valid signature (${request.signatureInput}), matching integrity check: loads.`);
    generate_fetch_test(request, {integrity:`ed25519-${kInvalidKey}`}, EXPECT_BLOCKED,
                        `Valid signature (${request.signatureInput}), mismatched integrity check: blocked.`);

    // <script>:
    generate_script_test(request, "", EXPECT_LOADED,
                        `Valid signature (${request.signatureInput}), no integrity check: loads.`);
    generate_script_test(request, `ed25519-${kValidKeys["rfc"]}`, EXPECT_LOADED,
                        `Valid signature (${request.signatureInput}), matching integrity check: loads.`);
    generate_script_test(request, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                        `Valid signature (${request.signatureInput}), mismatched integrity check: blocked.`);
}

// Test cases for failure
const kFailingRequests = [
    // Valid signature, but the "content-type" header value does not match what is signed.
    {
        body: kBody,
        digest: kDigest,
        signature: kSignatures.digest_then_ct,
        signatureInput: `signature=("unencoded-digest";sf "content-type");keyid="${kValidKeys["rfc"]}";tag="sri"`,
        type: "text/plain", // The signature was for "application/javascript"
        description: "Mismatched header value"
    },
    // Valid signature, but a signed header is missing from the response.
    {
        body: kBody,
        digest: kDigest,
        signature: kSignatures.with_extra_header,
        signatureInput: `signature=("content-type" "x-extra-header" "unencoded-digest";sf);keyid="${kValidKeys["rfc"]}";tag="sri"`,
        description: "Missing signed header"
    }
];

for (const request of kFailingRequests) {
    const description = request.description;
    // fetch():
    generate_fetch_test(request, {}, EXPECT_BLOCKED,
                        `${description}: blocked.`);
    generate_fetch_test(request, {integrity:`ed25519-${kValidKeys["rfc"]}`}, EXPECT_BLOCKED,
                        `${description} with matching integrity: blocked.`);

    // <script>:
    generate_script_test(request, "", EXPECT_BLOCKED,
                        `${description}: blocked.`);
    generate_script_test(request, `ed25519-${kValidKeys["rfc"]}`, EXPECT_BLOCKED,
                        `${description} with matching integrity: blocked.`);
}
