async function negotiate(offerer, answerer) {
  offerer.onicecandidate = e => answerer.addIceCandidate(e.candidate);
  answerer.onicecandidate = e => offerer.addIceCandidate(e.candidate);

  await offerer.setLocalDescription();
  await answerer.setRemoteDescription(offerer.localDescription);
  await answerer.setLocalDescription();
  await offerer.setRemoteDescription(answerer.localDescription);
}

// Calling this will only work if the channel is brand new
async function maybeWrapChannel(channel, shim = null) {
  if (shim) {
    assert_true(shim instanceof WorkerBackedDataChannel);
    await shim.init(channel);
    channel = shim;
  }
  await new Promise(r => channel.addEventListener("open", r, {once: true}));
  if (shim) {
    await shim.updateState();
  }
  return channel;
}

async function openChannelPair(pc1, pc2, label, config, workerShim1, workerShim2) {
  const channelEvent = new Promise(r => {
    pc2.addEventListener('datachannel', async ({channel}) => {
      if (channel.label == label) {
        channel = await maybeWrapChannel(channel, workerShim2);
        r(channel);
      }
    });
  });

  return Promise.all([
    maybeWrapChannel(pc1.createDataChannel(label, config), workerShim1),
    channelEvent
  ]);
}

async function makeDataChannelTestFixture(label, config, workerShim1, workerShim2) {
  const offerer = new RTCPeerConnection();
  const answerer = new RTCPeerConnection();
  const pairPromise = openChannelPair(
    offerer, answerer, label, {}, workerShim1, workerShim2);
  await negotiate(offerer, answerer);
  const [offererChannel, answererChannel] = await pairPromise;
  return {offerer, answerer, offererChannel, answererChannel};
}

async function openChannelPairOffererShimmed() {
  // Use worker shim in RTCDataChannel-worker-shim.js
  const {offerer, answerer, offererChannel, answererChannel} =
    await makeDataChannelTestFixture(
      'foo', {}, new WorkerBackedDataChannel(), null);

  return {
    shimmedChannel: offererChannel,
    nonShimmedChannel: answererChannel,
    offerer,
    answerer
  };
}

async function openChannelPairAnswererShimmed() {
  // Use worker shim in RTCDataChannel-worker-shim.js
  const {offerer, answerer, offererChannel, answererChannel} =
    await makeDataChannelTestFixture(
      'foo', {}, null, new WorkerBackedDataChannel());

  return {
    shimmedChannel: answererChannel,
    nonShimmedChannel: offererChannel,
    offerer,
    answerer
  };
}

async function openChannelPairWithShim(whichChannelShimmed) {
  switch (whichChannelShimmed) {
    case 'offerer':
      return openChannelPairOffererShimmed();
    case 'answerer':
      return openChannelPairAnswererShimmed();
  }
}
