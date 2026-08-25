// META: script=/common/utils.js
// META: script=resources/utils.js
//
// The test assumes the policy `Connection-Allowlist:
// "*://:subdomain.{{hosts[alt][]}}:*"` has been set in the response. The
// response also contains a link header that triggers a prefetch to the
// cross-origin KV server at http://{{hosts[alt][www]}}:{{ports[http][0]}}.

promise_test(async () => {
  const result =
      await nextValueFromServer('d5c4b2a1-7e8d-4c60-91da-2a57191363c2');
  assert_equals(result, 'hello');
}, 'Link header prefetch (as=document) to an allow-listed url succeeds.');
