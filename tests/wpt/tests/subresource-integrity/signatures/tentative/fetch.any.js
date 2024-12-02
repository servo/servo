// META: global=window,dedicatedworker,sharedworker

// Given `{ digest: "...", body: "...", cors: true, type: "..." }`:
function resourceURL(data) {
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

// HTTP/1.1 200 OK
// Date: Tue, 20 Apr 2021 02:07:56 GMT
// Content-Type: application/json
// Identity-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
// Content-Length: 18
// Signature-Input:
// signature=("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrI \
//           y29clF9CC/oPPsw3c5D0bs=";tag="sri"
// Signature: signature=:H7AqWWgo1DJ7VdyF9DKotG/4hvatKDfRTq2mpuY/hvJupSn+EYzus \
//            5p24qPK7DtVQcxJFhzSYDj4RBq9grZTAQ==:
//
// {"hello": "world"}
//
promise_test(test => {
  const data = {
    body: `{"hello": "world"}`
  };
  return fetch(resourceURL(data)).then(r => {
    assert_equals(r.status, 200, "Response status is 200.");
  });
}, "No signature: loads.");

promise_test(test => {
  const data = {
    body: `{"hello": "world"}`,
    digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
    signature: `signature=:H7AqWWgo1DJ7VdyF9DKotG/4hvatKDfRTq2mpuY/hvJupSn+EYzus5p24qPK7DtVQcxJFhzSYDj4RBq9grZTAQ==:`,
    signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
  };
  return fetch(resourceURL(data)).then(r => {
    assert_equals(r.status, 200, "Response status is 200.");
  });
}, "Valid signature: loads.");

promise_test(test => {
  const data = {
    body: `{"hello": "world"}`,
    digest: `sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:`,
    signature: `signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:`,
    signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"`
  };
  return promise_rejects_js(test, TypeError, fetch(resourceURL(data)));
}, "Non-matching signature: blocked.");
