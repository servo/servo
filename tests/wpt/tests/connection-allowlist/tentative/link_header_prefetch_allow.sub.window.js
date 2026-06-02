// META: script=/common/utils.js
// META: script=resources/utils.js
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a prefetch to the same-origin KV server at
// http://{{hosts[][]}}:{{ports[http][0]}}.

promise_test(async () => {
  const result =
      await nextValueFromServer('641530d4-9e7d-4760-91da-2a57191363c1');
  assert_equals(result, 'hello');
}, 'Link header prefetch to an allow-listed url succeeds.');
