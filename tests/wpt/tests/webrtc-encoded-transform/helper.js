'use strict';

// Returns an RTCEncodedVideoFrame or RTCEncodedAudioFrame
// on main thread. Spins up a peer connection and worker.
// Relies on serialization being implemented.

async function createRTCEncodedFrameFromScratch(kind) {
  function work() {
    onrtctransform = async ({transformer: {readable}}) => {
      try {
        const reader = readable.getReader();
        const {value, done} = await reader.read();
        if (done) throw {name: "done"};
        self.postMessage(value);
      } catch (e) {
        self.postMessage(e.name);
      }
    }
  }
  const worker = new Worker(`data:text/javascript,(${work.toString()})()`);
  const pc1 = new RTCPeerConnection(), pc2 = new RTCPeerConnection();
  pc1.onicecandidate = e => pc2.addIceCandidate(e.candidate);
  pc2.onicecandidate = e => pc1.addIceCandidate(e.candidate);
  pc1.onnegotiationneeded = async () => {
    await pc1.setLocalDescription(); await pc2.setRemoteDescription(pc1.localDescription);
    await pc2.setLocalDescription(); await pc1.setRemoteDescription(pc2.localDescription);
  }
  const stream = await getNoiseStream({[kind]: true});
  const sender = pc1.addTrack(stream.getTracks()[0], stream);
  sender.transform = new RTCRtpScriptTransform(worker);
  const {data} = await new Promise(r => worker.onmessage = r);
  return data;
}

function areArrayBuffersEqual(a, b) {
  if (a.byteLength != b.byteLength) {
    return false;
  }
  const ui8b = new Uint8Array(b);
  return new Uint8Array(a).every((val, i) => val === ui8b[i]);
}
