// META: script=/common/utils.js
// META: script=resources/utils.js
//
// The test assumes the policy `Connection-Allowlist: (response-origin)` has
// been set in the response. The response also contains a link header that
// triggers a modulepreload to the same-origin KV server at
// http://{{hosts[][]}}:{{ports[http][0]}}.

promise_test(async () => {
  const result =
      await nextValueFromServer('bb906791-f686-45f9-9bb7-4de2352ce382');
  assert_equals(result, 'hello');
}, 'Link header modulepreload to an allow-listed url succeeds.');
