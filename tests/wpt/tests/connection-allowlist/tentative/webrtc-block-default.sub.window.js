// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin)` has been set.
promise_test(async (t) => {
  try {
    const configuration = {};
    const peerConnection = new RTCPeerConnection(configuration);
    assert_unreached('RTCPeerConnection construction should fail.')
  } catch (err) {
    assert_equals(err.name, 'NotAllowedError');
  }
}, 'Test that setting Connection-Allowlist blocks WebRTC by default.');

promise_test(async (t) => {
  return fetch('/common/blank.html');
}, 'Fetches are unaffected by the `webrtc` property\'s value.');
