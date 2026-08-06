// META: script=/common/get-host-info.sub.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);webrtc=block` has been set.
//
// Other Connection-Allowlist tests for WebRTC are mostly interested in higher-
// level outcomes: whether the RTCPeerConnection transitions to a failure state,
// and whether violation reports are sent. These tests observe more specific
// script interactions with the RTCPeerConnection object if WebRTC has been
// blocked.
async function createBlockedPeerConnection() {
  // Copied from https://webrtc.org/getting-started/peer-connections.
  const configuration = {
    'iceServers': [{'urls': 'stun:stun.example.com:19302'}]
  };
  const pc1 = new RTCPeerConnection(configuration);
  const pc2 = new RTCPeerConnection(configuration);

  // Returns a promise which resolves to a boolean which is true
  // if and only if pc.iceConnectionState settles in the "failed"
  // state, and never transitions to any state other than "new"
  // or "failed."
  const pcFailed =
      (pc) => {
        return new Promise((resolve, _reject) => {
          pc.oniceconnectionstatechange = (e) => {
            resolve(pc.iceConnectionState == 'failed');
          };
        });
      }

  let pc1Failed = pcFailed(pc1);
  let pc2Failed = pcFailed(pc2);

  // Creating a data channel is necessary to induce negotiation:
  const channel = pc1.createDataChannel('test');

  // Usual webrtc signaling dance:
  pc1.onicecandidate = ({candidate}) => pc2.addIceCandidate(candidate);
  pc2.onicecandidate = ({candidate}) => pc1.addIceCandidate(candidate);
  const offer = await pc1.createOffer();
  await pc1.setLocalDescription(offer);
  await pc2.setRemoteDescription(pc1.localDescription);
  const answer = await pc2.createAnswer();
  await pc2.setLocalDescription(answer);
  await pc1.setRemoteDescription(pc2.localDescription);

  const failed1 = await pc1Failed;
  const failed2 = await pc2Failed;
  assert_true(failed1);
  assert_true(failed2);
  return pc1;
};

promise_test(async (t) => {
  let pc = await createBlockedPeerConnection();

  // Even though we passed an ICE server into the RTCPeerConnection's
  // constructor, it should have been filtered out before any requests could be
  // made to it.
  assert_equals(pc.getConfiguration().iceServers.length, 0);
}, 'ICE servers are not present when WebRTC is blocked.');

promise_test(async (t) => {
  let pc = await createBlockedPeerConnection();

  // Adding an ICE candidate to the connection should return an empty promise.
  let candidate = new RTCIceCandidate({
    sdpMid: 'video',
    sdpMLineIndex: 1,
    usernameFragment: 'test',
    relayProtocol: 'udp',
    url: 'stun:stun.example.org'
  });
  let candidateResult = await pc.addIceCandidate(candidate);
  assert_equals(candidateResult, undefined);
}, 'Adding a candidate returns undefined when WebRTC is blocked.');

promise_test(async (t) => {
  let pc = await createBlockedPeerConnection();

  let iceStatePromise = new Promise((resolve, _reject) => {
    pc.oniceconnectionstatechange = (e) => {
      resolve('Ice state changed.');
    };
  });
  pc.restartIce();

  let result = await Promise.race(
      [new Promise(r => t.step_timeout(r, 2000)), iceStatePromise]);
  assert_equals(result, undefined);
}, 'restartIce() has no observable effect when WebRTC is blocked.');
