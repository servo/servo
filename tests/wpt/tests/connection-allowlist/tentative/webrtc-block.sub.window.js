// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);webrtc=block` has been set.
promise_test(async (t) => {
  try {
    // Copied from https://webrtc.org/getting-started/peer-connections.
    const configuration = {
      'iceServers': [{'urls': 'stun:stun.example.com:19302'}]
    };
    const peerConnection = new RTCPeerConnection(configuration);
    assert_unreached('RTCPeerConnection construction should fail.')
  } catch (err) {
    assert_equals(err.name, 'NotAllowedError');
  }
}, 'Test that webrtc=block Connection-Allowlist param is respected.');

promise_test(async (t) => {
  return fetch('/common/blank.html');
}, 'Fetches are unaffected by the `webrtc` property\'s value.');
