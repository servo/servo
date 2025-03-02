// META: script=helper.js

// The following tests validate the behavior of the `@query` derived component.
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
// Unencoded-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
// Content-Length: 18
// Signature-Input: signature=("unencoded-digest";sf "@query";req); \
//                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";       \
//                  tag="sri"
// Signature: signature=[SEE NOTE BELOW]
//
// {"hello": "world"}
// ```
//
// Unlike other tests in this directory, we cannot pass the signature into the resource,
// as it would then be reflected via the signature base's inclusion of the query string.
// Instead, `query-response.py` contains a hard-coded set of signatures for the test
// cases below.


let test_cases = [
  "", "test", "test=a", "test=%2F",  "test=ü"
];

for (let query of test_cases) {
  generate_query_test(query, {}, EXPECT_LOADED,
                      `Query = "${query}": no integrity check: loads.`);
  generate_query_test(query, {integrity:`ed25519-${kValidKeys['rfc']}`}, EXPECT_LOADED,
                      `Query = "${query}": matching integrity check: loads.`);
  generate_query_test(query, {integrity:`ed25519-${kInvalidKey}`}, EXPECT_BLOCKED,
                      `Query = "${query}": mismatched integrity check: blocked.`);
}
