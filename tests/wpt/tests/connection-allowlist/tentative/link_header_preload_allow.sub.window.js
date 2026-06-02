// META: script=/common/utils.js
// META: script=resources/utils.js
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a preload to the same-origin KV server at
// http://{{hosts[][]}}:{{ports[http][0]}}.

promise_test(async () => {
  const result =
      await nextValueFromServer('6cd381d3-85b5-40ce-8c63-1ffb06154b8b');
  assert_equals(result, 'hello');
}, 'Link header preload to an allow-listed url succeeds.');
