// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a modulepreload to the cross-origin KV server at
// http://{{hosts[][www]}}:{{ports[http][0]}}.

promise_test(async (t) => {
  const result = await Promise.race([
    new Promise(r => t.step_timeout(r, 1000)),
    nextValueFromServer('cabcb250-9769-4d6a-976a-8e8aa6a95f6a')
  ]);
  assert_true(typeof result === 'undefined');
}, 'Link header modulepreload to a not allow-listed url fails.');
