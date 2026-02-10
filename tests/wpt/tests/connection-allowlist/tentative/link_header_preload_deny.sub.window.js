// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a preload to the cross-origin KV server at
// http://{{hosts[][www]}}:{{ports[http][0]}}.

promise_test(async (t) => {
  const result = await Promise.race([
    new Promise(r => t.step_timeout(r, 1000)),
    nextValueFromServer('277a5831-dd75-4d34-bf27-975a8ede398e')
  ]);
  assert_true(typeof result === 'undefined');
}, 'Link header preload to a not allow-listed url fails.');
