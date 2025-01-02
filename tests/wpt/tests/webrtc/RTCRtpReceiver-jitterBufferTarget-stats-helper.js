async function measureDelayFromStats(t, receiver, cycles, targetDelay, tolerance) {
  let oldInboundStats;

  for (let i = 0; i < cycles; i++) {
    const statsReport = await receiver.getStats();
    const inboundStats = [...statsReport.values()].find(({type}) => type == "inbound-rtp");

    if (inboundStats) {
      if (oldInboundStats) {
        const emittedCount = inboundStats.jitterBufferEmittedCount - oldInboundStats.jitterBufferEmittedCount;

        if (emittedCount) {
          const delay = 1000 * (inboundStats.jitterBufferDelay - oldInboundStats.jitterBufferDelay) / emittedCount;

          if (Math.abs(delay - targetDelay) < tolerance) {
            return true;
          }
        }
      }
      oldInboundStats = inboundStats;
    }
    await new Promise(r => t.step_timeout(r, 1000));
  }

  return false;
}

async function applyJitterBufferTarget(t, kind, target) {
  const caller = new RTCPeerConnection();
  t.add_cleanup(() => caller.close());
  const callee = new RTCPeerConnection();
  t.add_cleanup(() => callee.close());

  const stream = await getNoiseStream({[kind]:true});
  t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
  caller.addTransceiver(stream.getTracks()[0], {streams: [stream]});

  exchangeIceCandidates(caller, callee);
  await exchangeOffer(caller, callee);
  await exchangeAnswer(caller, callee);

  const receiver = callee.getReceivers()[0];

  // Workaround for Chromium to pull audio from jitter buffer.
  if (kind === "audio") {
    const audio = document.createElement("audio");

    audio.srcObject = new MediaStream([receiver.track]);
    audio.play();
  }
  assert_equals(receiver.jitterBufferTarget, null,
   `jitterBufferTarget supported for ${kind}`);

  let result = await measureDelayFromStats(t, receiver, 5, 0, 100);
  assert_true(result, 'jitter buffer is not stabilised');

  receiver.jitterBufferTarget = target;
  assert_equals(receiver.jitterBufferTarget, target,
    `jitterBufferTarget increase target for ${kind}`);

  result = await measureDelayFromStats(t, receiver, 10, target, 20);
  assert_true(result, 'jitterBuffer does not reach target');

  receiver.jitterBufferTarget = 0;
  assert_equals(receiver.jitterBufferTarget, 0,
      `jitterBufferTarget decrease target for ${kind}`);

  result = await measureDelayFromStats(t, receiver, 10, 0, 100);
  assert_true(result, 'jitter buffer delay is not back to normal');
}
