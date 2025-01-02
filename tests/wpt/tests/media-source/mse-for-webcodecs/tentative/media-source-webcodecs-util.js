const testIV = window.crypto.getRandomValues(new Uint8Array(16));
const testKey = window.crypto.getRandomValues(new Uint8Array(16));
const testKeyId = window.crypto.getRandomValues(new Uint8Array(8));
var testEncodedKey = null;

const keySystemConfig = [{
  initDataTypes: ['keyids'],
  videoCapabilities: [{contentType: 'video/mp4; codecs="vp09.00.10.08"'}]
}];

// TODO(crbug.com/1144908): Consider extracting metadata into helper library
// shared with webcodecs tests. This metadata is adapted from
// webcodecs/video-decoder-any.js.
const vp9 = {
  async buffer() {
    return (await fetch('vp9.mp4')).arrayBuffer();
  },
  // Note, file might not actually be level 1. See original metadata in
  // webcodecs test suite.
  codec: 'vp09.00.10.08',
  frames: [
    {offset: 44, size: 3315, type: 'key'},
    {offset: 3359, size: 203, type: 'delta'},
    {offset: 3562, size: 245, type: 'delta'},
    {offset: 3807, size: 172, type: 'delta'},
    {offset: 3979, size: 312, type: 'delta'},
    {offset: 4291, size: 170, type: 'delta'},
    {offset: 4461, size: 195, type: 'delta'},
    {offset: 4656, size: 181, type: 'delta'},
    {offset: 4837, size: 356, type: 'delta'},
    {offset: 5193, size: 159, type: 'delta'}
  ]
};

async function getOpenMediaSource(t) {
  return new Promise(async resolve => {
    const v = document.createElement('video');
    document.body.appendChild(v);
    const mediaSource = new MediaSource();
    const url = URL.createObjectURL(mediaSource);
    mediaSource.addEventListener(
        'sourceopen', t.step_func(() => {
          URL.revokeObjectURL(url);
          assert_equals(mediaSource.readyState, 'open', 'MediaSource is open');
          resolve([v, mediaSource]);
        }),
        {once: true});
    v.src = url;
  });
}

async function setupEme(t, video) {
  testEncodedKey = await crypto.subtle.importKey(
      'raw', testKey.buffer, 'AES-CTR', false, ['encrypt', 'decrypt']);

  var handler = new MessageHandler(
      'org.w3.clearkey', {keys: [{kid: testKeyId, key: testKey}]});

  function handleMessage(event) {
    handler.messagehandler(event.messageType, event.message).then(response => {
      event.target.update(response).catch(e => {
        assert_unreached('Failed to update session: ' + e);
      });
    });
  }

  return navigator
      .requestMediaKeySystemAccess('org.w3.clearkey', keySystemConfig)
      .then(keySystemAccess => {
        return keySystemAccess.createMediaKeys();
      })
      .then(createdMediaKeys => {
        return video.setMediaKeys(createdMediaKeys);
      })
      .then(_ => {
        let session = video.mediaKeys.createSession();
        session.addEventListener('message', handleMessage, false);

        let encoder = new TextEncoder();
        let initData = encoder.encode(
            JSON.stringify({'kids': [base64urlEncode(testKeyId)]}));
        session.generateRequest('keyids', initData).catch(e => {
          assert_unreached('Failed to generate a license request: ' + e);
        });
      })
      .catch(e => {
        assert_unreached('Failed to setup EME: ', e);
      });
}

async function runEncryptedChunksTest(t) {
  let buffer = await vp9.buffer();
  let [videoElement, mediaSource] = await getOpenMediaSource(t);

  // Makes early prototype demo playback easier to control manually.
  videoElement.controls = true;

  await setupEme(t, videoElement);

  let sourceBuffer = mediaSource.addSourceBuffer(
      {videoConfig: {codec: vp9.codec, encryptionScheme: 'cenc'}});
  let nextTimestamp = 0;
  let frameDuration = 100 * 1000;  // 100 milliseconds
  // forEach with async callbacks makes it too easy to have uncaught rejections
  // that don't fail this promise_test or even emit harness error.
  // Iterating explicitly instead.
  for (i = 0; i < vp9.frames.length; i++, nextTimestamp += frameDuration) {
    let frameMetadata = vp9.frames[i];
    let frameData =
        new Uint8Array(buffer, frameMetadata.offset, frameMetadata.size);
    let encryptedFrameData = await window.crypto.subtle.encrypt(
        {name: 'AES-CTR', counter: testIV, length: 128}, testEncodedKey,
        frameData);

    await sourceBuffer.appendEncodedChunks(new EncodedVideoChunk({
      type: frameMetadata.type,
      timestamp: nextTimestamp,
      duration: frameDuration,
      data: encryptedFrameData,
      decryptConfig: {
        encryptionScheme: 'cenc',
        keyId: testKeyId,
        initializationVector: testIV,
        subsampleLayout: [{clearBytes: 0, cypherBytes: frameMetadata.size}],
      }
    }));
  }

  mediaSource.endOfStream();

  return new Promise((resolve, reject) => {
    videoElement.onended = resolve;
    videoElement.onerror = reject;
    videoElement.play();
  });
}
