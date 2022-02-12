'use strict'

function peer(other, polite, fail = null) {
  const send = (tgt, msg) => tgt.postMessage(JSON.parse(JSON.stringify(msg)),
                                             "*");
  if (!fail) fail = e => send(window.parent, {error: `${e.name}: ${e.message}`});
  const pc = new RTCPeerConnection();

  if (!window.assert_equals) {
    window.assert_equals = (a, b, msg) => a === b ||
        fail(new Error(`${msg} expected ${b} but got ${a}`));
  }

  const commands = {
    async addTransceiver() {
      const transceiver = pc.addTransceiver("video");
      await new Promise(r => pc.addEventListener("negotiated", r, {once: true}));
      if (!transceiver.currentDirection) {
        // Might have just missed the negotiation train. Catch next one.
        await new Promise(r => pc.addEventListener("negotiated", r, {once: true}));
      }
      assert_equals(transceiver.currentDirection, "sendonly", "have direction");
      return pc.getTransceivers().length;
    },
    async simpleConnect() {
      const p = commands.addTransceiver();
      await new Promise(r => pc.oniceconnectionstatechange =
                        () => pc.iceConnectionState == "connected" && r());
      return await p;
    },
    async getNumTransceivers() {
      return pc.getTransceivers().length;
    },
  };

  try {
    pc.addEventListener("icecandidate", ({candidate}) => send(other,
                                                              {candidate}));
    let makingOffer = false, ignoreIceCandidateFailures = false;
    let srdAnswerPending = false;
    pc.addEventListener("negotiationneeded", async () => {
      try {
        assert_equals(pc.signalingState, "stable", "negotiationneeded always fires in stable state");
        assert_equals(makingOffer, false, "negotiationneeded not already in progress");
        makingOffer = true;
        await pc.setLocalDescription();
        assert_equals(pc.signalingState, "have-local-offer", "negotiationneeded not racing with onmessage");
        assert_equals(pc.localDescription.type, "offer", "negotiationneeded SLD worked");
        send(other, {description: pc.localDescription});
      } catch (e) {
        fail(e);
      } finally {
        makingOffer = false;
      }
    });
    window.onmessage = async ({data: {description, candidate, run}}) => {
      try {
        if (description) {
          // If we have a setRemoteDescription() answer operation pending, then
          // we will be "stable" by the time the next setRemoteDescription() is
          // executed, so we count this being stable when deciding whether to
          // ignore the offer.
          let isStable =
              pc.signalingState == "stable" ||
              (pc.signalingState == "have-local-offer" && srdAnswerPending);
          const ignoreOffer = description.type == "offer" && !polite &&
                         (makingOffer || !isStable);
          if (ignoreOffer) {
            ignoreIceCandidateFailures = true;
            return;
          }
          if (description.type == "answer")
            srdAnswerPending = true;
          await pc.setRemoteDescription(description);
          ignoreIceCandidateFailures = false;
          srdAnswerPending = false;
          if (description.type == "offer") {
            assert_equals(pc.signalingState, "have-remote-offer", "Remote offer");
            assert_equals(pc.remoteDescription.type, "offer", "SRD worked");
            await pc.setLocalDescription();
            assert_equals(pc.signalingState, "stable", "onmessage not racing with negotiationneeded");
            assert_equals(pc.localDescription.type, "answer", "onmessage SLD worked");
            send(other, {description: pc.localDescription});
          } else {
            assert_equals(pc.remoteDescription.type, "answer", "Answer was set");
            assert_equals(pc.signalingState, "stable", "answered");
            pc.dispatchEvent(new Event("negotiated"));
          }
        } else if (candidate) {
          try {
            await pc.addIceCandidate(candidate);
          } catch (e) {
            if (!ignoreIceCandidateFailures) throw e;
          }
        } else if (run) {
          send(window.parent, {[run.id]: await commands[run.cmd]() || 0});
        }
      } catch (e) {
        fail(e);
      }
    };
  } catch (e) {
    fail(e);
  }
  return pc;
}

async function setupPeerIframe(t, polite) {
  const iframe = document.createElement("iframe");
  t.add_cleanup(() => iframe.remove());
  iframe.srcdoc =
   `<html\><script\>(${peer.toString()})(window.parent, ${polite});</script\></html\>`;
  document.documentElement.appendChild(iframe);

  const failCatcher = t.step_func(({data}) =>
      ("error" in data) && assert_unreached(`Error in iframe: ${data.error}`));
  window.addEventListener("message", failCatcher);
  t.add_cleanup(() => window.removeEventListener("message", failCatcher));
  await new Promise(r => iframe.onload = r);
  return iframe;
}

function setupPeerTopLevel(t, other, polite) {
  const pc = peer(other, polite, t.step_func(e => { throw e; }));
  t.add_cleanup(() => { pc.close(); window.onmessage = null; });
}

let counter = 0;
async function run(target, cmd) {
  const id = `result${counter++}`;
  target.postMessage({run: {cmd, id}}, "*");
  return new Promise(r => window.addEventListener("message",
                                                  function listen({data}) {
    if (!(id in data)) return;
    window.removeEventListener("message", listen);
    r(data[id]);
  }));
}

let iframe;
async function setupAB(t, politeA, politeB) {
  iframe = await setupPeerIframe(t, politeB);
  return setupPeerTopLevel(t, iframe.contentWindow, politeA);
}
const runA = cmd => run(window, cmd);
const runB = cmd => run(iframe.contentWindow, cmd);
const runBoth = (cmdA, cmdB = cmdA) => Promise.all([runA(cmdA), runB(cmdB)]);

async function promise_test_both_roles(f, name) {
  promise_test(async t => f(t, await setupAB(t, true, false)), name);
  promise_test(async t => f(t, await setupAB(t, false, true)),
               `${name} with roles reversed`);
}
