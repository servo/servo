// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);webrtc=allow` has been set.
promise_test(async (t) => {
  try {
    const configuration = {};
    const peerConnection = new RTCPeerConnection(configuration);
  } catch (err) {
    assert_unreached('RTCPeerConnection construction should succeed');
  }
}, 'Test that webrtc=allow Connection-Allowlist param is respected.');

promise_test(async (t) => {
  return fetch('/common/blank.html');
}, 'Fetches are unaffected by the `webrtc` property\'s value.');
