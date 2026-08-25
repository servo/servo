// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a prefetch to the cross-origin KV server at
// http://{{hosts[][www]}}:{{ports[http][0]}}.

promise_test(async (t) => {
  const result = await Promise.race([
    new Promise(r => t.step_timeout(r, 1000)),
    nextValueFromServer('639d1a56-16f6-4f1d-8739-87b1278441e6')
  ]);
  assert_true(typeof result === 'undefined');
}, 'Link header prefetch to a not allow-listed url fails.');
