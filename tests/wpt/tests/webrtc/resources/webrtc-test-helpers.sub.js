// SDP copied from JSEP Example 7.1
// It contains two media streams with different ufrags
// to test if candidate is added to the correct stream
const sdp = `v=0
o=- 4962303333179871722 1 IN IP4 0.0.0.0
s=-
t=0 0
a=ice-options:trickle
a=group:BUNDLE a1 v1
a=group:LS a1 v1
m=audio 10100 UDP/TLS/RTP/SAVPF 96 0 8 97 98
c=IN IP4 203.0.113.100
a=mid:a1
a=sendrecv
a=rtpmap:96 opus/48000/2
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
a=rtpmap:97 telephone-event/8000
a=rtpmap:98 telephone-event/48000
a=maxptime:120
a=extmap:1 urn:ietf:params:rtp-hdrext:sdes:mid
a=extmap:2 urn:ietf:params:rtp-hdrext:ssrc-audio-level
a=msid:47017fee-b6c1-4162-929c-a25110252400 f83006c5-a0ff-4e0a-9ed9-d3e6747be7d9
a=ice-ufrag:ETEn
a=ice-pwd:OtSK0WpNtpUjkY4+86js7ZQl
a=fingerprint:sha-256 19:E2:1C:3B:4B:9F:81:E6:B8:5C:F4:A5:A8:D8:73:04:BB:05:2F:70:9F:04:A9:0E:05:E9:26:33:E8:70:88:A2
a=setup:actpass
a=dtls-id:1
a=rtcp:10101 IN IP4 203.0.113.100
a=rtcp-mux
a=rtcp-rsize
m=video 10102 UDP/TLS/RTP/SAVPF 100 101
c=IN IP4 203.0.113.100
a=mid:v1
a=sendrecv
a=rtpmap:100 VP8/90000
a=rtpmap:101 rtx/90000
a=fmtp:101 apt=100
a=extmap:1 urn:ietf:params:rtp-hdrext:sdes:mid
a=rtcp-fb:100 ccm fir
a=rtcp-fb:100 nack
a=rtcp-fb:100 nack pli
a=msid:47017fee-b6c1-4162-929c-a25110252400 f30bdb4a-5db8-49b5-bcdc-e0c9a23172e0
a=ice-ufrag:BGKk
a=ice-pwd:mqyWsAjvtKwTGnvhPztQ9mIf
a=fingerprint:sha-256 19:E2:1C:3B:4B:9F:81:E6:B8:5C:F4:A5:A8:D8:73:04:BB:05:2F:70:9F:04:A9:0E:05:E9:26:33:E8:70:88:A2
a=setup:actpass
a=dtls-id:1
a=rtcp:10103 IN IP4 203.0.113.100
a=rtcp-mux
a=rtcp-rsize
`;

const sessionDesc = { type: 'offer', sdp };
const candidate = {
  candidate: 'candidate:1 1 udp 2113929471 203.0.113.100 10100 typ host',
  sdpMid: 'a1',
  sdpMLineIndex: 0,
  usernameFragment: 'ETEn'
};

// Opens a new WebRTC connection.
async function openWebRTC(remoteContextHelper) {
  await remoteContextHelper.executeScript(async (sessionDesc, candidate) => {
    window.testRTCPeerConnection = new RTCPeerConnection();
    await window.testRTCPeerConnection.setRemoteDescription(sessionDesc);
    await window.testRTCPeerConnection.addIceCandidate(candidate);
  }, [sessionDesc, candidate]);
}

// Opens a new WebRTC connection and then close it.
async function openThenCloseWebRTC(remoteContextHelper) {
  await remoteContextHelper.executeScript(async (sessionDesc, candidate) => {
    window.testRTCPeerConnection = new RTCPeerConnection();
    await window.testRTCPeerConnection.setRemoteDescription(sessionDesc);
    await window.testRTCPeerConnection.addIceCandidate(candidate);
    window.testRTCPeerConnection.close();
  }, [sessionDesc, candidate]);
}
