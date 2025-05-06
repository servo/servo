// META: script=/common/get-host-info.sub.js
// META: script=helper.js

// The following tests validate the behavior of the `@scheme` derived component.
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
// Unencoded-Digest: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
// Content-Length: 18
// Signature-Input: signature=("unencoded-digest";sf "@scheme";req); \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";       \
//                  tag="sri"
// Signature: signature=:oVQ+s/OqXLAVdfvgZ3HaPiyzkpNXZSit9l6e1FB/gOOL3t8FOrIRDV \
//                       CkcIEcJjd3MA1mROn39/WQShTmnKmlDg==:
//
//
// window.hello = `world`;
// ```

const test_cases = [
  // ```
  // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@scheme";req: http
  // "@signature-params": ("unencoded-digest";sf "@scheme";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  // ```
  {
    origin: get_host_info().HTTP_REMOTE_ORIGIN,
    signature: `signature=:WZp87p7X3ELfgIKL/qxsY/CT6XArMvZRaxcJ3uy1QklEcLf0c8tol2+W2pvaXX4jnd7hGevFVkzWE77rCOIzAA==:`,
  },
  // ```
  // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
  // "@scheme";req: https
  // "@signature-params": ("unencoded-digest";sf "@scheme";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
  // ```
  {
    origin: get_host_info().HTTPS_REMOTE_ORIGIN,
    signature: `signature=:lMzR8lIXYG0Iz0MmTXcRTcBfNw6TgBAPfaNLAU1LzsxWC5dlez8SNe7aCW7avHTWKgaqTGBCMW1LgxkHlijgDA==:`,
  }
]

// Valid signatures depend upon integrity checks.
//
// We're testing our handling of malformed and multiple keys generally in
// the broader `client-initiated.*` tests. Here we'll just focus on ensuring
// that responses with `@scheme` components load at all (no integrity check),
// load when integrity checks match, and fail when integrity checks mismatch.
for (const test_case of test_cases) {
    const request = {
      cors: true,
      body: "window.hello = `world`;",
      digest: "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:",
      signatureInput: `signature=("unencoded-digest";sf "@scheme";req);keyid="${kValidKeys['rfc']}";tag="sri"`,
      signature: test_case.signature
    };

    // fetch():
    generate_fetch_test(request, {origin: test_case.origin}, EXPECT_LOADED,
                        `Valid signature (${request.signature}), no integrity check: loads.`);
    generate_fetch_test(request, {origin: test_case.origin,
                                  integrity:`ed25519-${kValidKeys['rfc']}`}, EXPECT_LOADED,
                        `Valid signature (${request.signature}), matching integrity check: loads.`);

    generate_fetch_test(request, {origin: test_case.origin,
                                  integrity:`ed25519-${kInvalidKey}`}, EXPECT_BLOCKED,
                        `Valid signature (${request.signature}), mismatched integrity check: blocked.`);
}
