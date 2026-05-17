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
    nextValueFromServer('a9853747-3a23-4802-a907-327ce0a888f0')
  ]);
  assert_true(typeof result === 'undefined');
}, 'Link header prefetch (as=document) to a not allow-listed url fails.');
