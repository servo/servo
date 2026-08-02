// META: script=/common/get-host-info.sub.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin)` has been set. WebRTC should be blocked by default.
promise_test(async (t) => {
  assert_equals(await tryConnect(), 'blocked');
}, 'Test that default Connection-Allowlist WebRTC blocking is respected.');

promise_test(async (t) => {
  return fetch('/common/blank.html');
}, 'Fetches are unaffected by the `webrtc` property\'s value.');
