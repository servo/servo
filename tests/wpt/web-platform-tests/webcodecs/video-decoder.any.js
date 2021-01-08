// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

// TODO(sandersd): Move metadata into a helper library.
// TODO(sandersd): Add H.264 decode test once there is an API to query for
// supported codecs.
const h264 = {
  async buffer() { return (await fetch('h264.mp4')).arrayBuffer(); },
  codec: "avc1.64000c",
  description: {offset: 7229, size: 46},
  frames: [{offset: 48, size: 4007},
           {offset: 4055, size: 926},
           {offset: 4981, size: 241},
           {offset: 5222, size: 97},
           {offset: 5319, size: 98},
           {offset: 5417, size: 624},
           {offset: 6041, size: 185},
           {offset: 6226, size: 94},
           {offset: 6320, size: 109},
           {offset: 6429, size: 281}]
};

const vp9 = {
  async buffer() { return (await fetch('vp9.mp4')).arrayBuffer(); },
  // TODO(sandersd): Verify that the file is actually level 1.
  codec: "vp09.00.10.08",
  frames: [{offset: 44, size: 3315},
           {offset: 3359, size: 203},
           {offset: 3562, size: 245},
           {offset: 3807, size: 172},
           {offset: 3979, size: 312},
           {offset: 4291, size: 170},
           {offset: 4461, size: 195},
           {offset: 4656, size: 181},
           {offset: 4837, size: 356},
           {offset: 5193, size: 159}]
};

const badCodecsList = [
    '',                         // Empty codec
    'bogus',                    // Non exsitent codec
    'vorbis',                   // Audio codec
    'vp9',                      // Ambiguous codec
    'video/webm; codecs="vp9"'  // Codec with mime type
  ]

const invalidConfigs = [
  {
    comment: 'Emtpy codec',
    config: {codec: ''},
  },
  {
    comment: 'Unrecognized codec',
    config: {codec: 'bogus'},
  },
  {
    comment: 'Audio codec',
    config: {codec: 'vorbis'},
  },
  {
    comment: 'Ambiguous codec',
    config: {codec: 'vp9'},
  },
  {
    comment: 'Codec with MIME type',
    config: {codec: 'video/webm; codecs="vp8"'},
  },
  {
    comment: 'Zero coded size',
    config: {
      codec: h264.codec,
      codedWidth: 0,
      codedHeight: 0,
    },
  },
  {
    comment: 'Out of bounds crop size caused by left/top offset',
    config: {
      codec: h264.codec,
      codedWidth: 1920,
      codedHeight: 1088,
      cropLeft: 10,
      cropTop: 10,
      // When unspecified, these default to coded dimensions
      // cropWidth: 1920,
      // cropHeight: 1088
    },
  },
  {
    comment: 'Out of bounds crop size',
    config: {
      codec: h264.codec,
      codedWidth: 1920,
      codedHeight: 1088,
      cropLeft: 10,
      cropTop: 10,
      cropWidth: 1920,
      cropHeight: 1088,
    },
  },
  {
    comment: 'Way out of bounds crop size',
    config: {
      codec: h264.codec,
      codedWidth: 1920,
      codedHeight: 1088,
      cropWidth: 4000,
      cropHeight: 5000,
    },
  },
  {
    comment: 'Invalid display size',
    config: {
      codec: h264.codec,
      displayWidth: 0,
      displayHeight: 0,
    },
  },
] //  invalidConfigs

function view(buffer, {offset, size}) {
  return new Uint8Array(buffer, offset, size);
}

function getFakeChunk() {
  return new EncodedVideoChunk({
    type:'key',
    timestamp:0,
    data:Uint8Array.of(0)
  });
}

invalidConfigs.forEach(entry => {
  promise_test(t => {
    return promise_rejects_js(t, TypeError, VideoDecoder.isConfigSupported(entry.config));
  }, 'Test that VideoDecoder.isConfigSupported() rejects invalid config:' + entry.comment);
});

invalidConfigs.forEach(entry => {
  async_test(t => {
    let codec = new VideoDecoder(getDefaultCodecInit(t));
    assert_throws_js(TypeError, () => { codec.configure(entry.config); });
    t.done();
  }, 'Test that VideoDecoder.configure() rejects invalid config:' + entry.comment);
});

promise_test(t => {
  return VideoDecoder.isConfigSupported({codec: vp9.codec});
}, 'Test VideoDecoder.isConfigSupported() with minimal valid config');

promise_test(t => {
  // This config specifies a slight crop. H264 1080p content always crops
  // because H264 coded dimensions are a multiple of 16 (e.g. 1088).
  return VideoDecoder.isConfigSupported({
    codec: h264.codec,
    codedWidth: 1920,
    codedHeight: 1088,
    cropLeft: 0,
    cropTop: 0,
    cropWidth: 1920,
    cropHeight: 1080,
    displayWidth: 1920,
    displayHeight: 1080
  });
}, 'Test VideoDecoder.isConfigSupported() with valid expanded config');

promise_test(t => {
  // VideoDecoderInit lacks required fields.
  assert_throws_js(TypeError, () => { new VideoDecoder({}); });

  // VideoDecoderInit has required fields.
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  assert_equals(decoder.state, "unconfigured");

  decoder.close();

  return endAfterEventLoopTurn();
}, 'Test VideoDecoder construction');

promise_test(t => {
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  // TODO(chcunningham): Remove badCodecsList testing. It's now covered more
  // extensively by other tests.
  testConfigurations(decoder, { codec: vp9.codec }, badCodecsList);

  return endAfterEventLoopTurn();
}, 'Test VideoDecoder.configure() with various codec strings');

promise_test(async t => {
  let buffer = await vp9.buffer();

  let numOutputs = 0;
  let decoder = new VideoDecoder({
    output(frame) {
      t.step(() => {
        assert_equals(++numOutputs, 1, "outputs");
        assert_equals(frame.cropWidth, 320, "cropWidth");
        assert_equals(frame.cropHeight, 240, "cropHeight");
        assert_equals(frame.timestamp, 0, "timestamp");
        frame.destroy();
      });
    },
    error(e) {
      t.step(() => { throw e; });
    }
  });

  decoder.configure({codec: vp9.codec});

  decoder.decode(new EncodedVideoChunk({
    type:'key',
    timestamp:0,
    data: view(buffer, vp9.frames[0])
  }));

  await decoder.flush();

  assert_equals(numOutputs, 1, "outputs");
}, 'Decode VP9');

promise_test(async t => {
  let buffer = await vp9.buffer();

  let outputs_before_reset = 0;
  let outputs_after_reset = 0;

  let decoder = new VideoDecoder({
    // Pre-reset() chunks will all have timestamp=0, while post-reset() chunks
    // will all have timestamp=1.
    output(frame) {
      t.step(() => {
        if (frame.timestamp == 0)
          outputs_before_reset++;
        else
          outputs_after_reset++;
      });
    },
    error(e) {
      t.step(() => { throw e; });
    }
  });

  decoder.configure({codec: vp9.codec});

  for (let i = 0; i < 100; i++) {
    decoder.decode(new EncodedVideoChunk({
      type:'key',
      timestamp:0,
      data: view(buffer, vp9.frames[0])
    }));
  }

  assert_greater_than(decoder.decodeQueueSize, 0);

  // Wait for the first frame to be decoded.
  await t.step_wait(() => outputs_before_reset > 0,
      "Decoded outputs started coming", 10000, 1);

  let saved_outputs_before_reset = outputs_before_reset;
  assert_greater_than(saved_outputs_before_reset, 0);
  assert_less_than(saved_outputs_before_reset, 100);

  decoder.reset()
  assert_equals(decoder.decodeQueueSize, 0);

  decoder.configure({codec: vp9.codec});

  for (let i = 0; i < 5; i++) {
    decoder.decode(new EncodedVideoChunk({
      type:'key',
      timestamp:1,
      data: view(buffer, vp9.frames[0])
    }));
  }
  await decoder.flush();
  assert_equals(outputs_after_reset, 5);
  assert_equals(saved_outputs_before_reset, outputs_before_reset);
  assert_equals(decoder.decodeQueueSize, 0);

  endAfterEventLoopTurn();
}, 'Verify reset() suppresses output and rejects flush');

promise_test(t => {
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  return testClosedCodec(t, decoder, { codec: vp9.codec }, getFakeChunk());
}, 'Verify closed VideoDecoder operations');

promise_test(t => {
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  return testUnconfiguredCodec(t, decoder, getFakeChunk());
}, 'Verify unconfigured VideoDecoder operations');

promise_test(t => {
  let numErrors = 0;
  let codecInit = getDefaultCodecInit(t);
  codecInit.error = _ => numErrors++;

  let decoder = new VideoDecoder(codecInit);

  decoder.configure({codec: vp9.codec});

  let fakeChunk = getFakeChunk();
  decoder.decode(fakeChunk);

  return promise_rejects_exactly(t, undefined, decoder.flush()).then(
      () => {
        assert_equals(numErrors, 1, "errors");
        assert_equals(decoder.state, "closed");
      });
}, 'Decode corrupt VP9 frame');

promise_test(t => {
  let numErrors = 0;
  let codecInit = getDefaultCodecInit(t);
  codecInit.error = _ => numErrors++;

  let decoder = new VideoDecoder(codecInit);

  decoder.configure({codec: vp9.codec});

  let fakeChunk = getFakeChunk();
  decoder.decode(fakeChunk);

  return promise_rejects_exactly(t, undefined, decoder.flush()).then(
      () => {
        assert_equals(numErrors, 1, "errors");
        assert_equals(decoder.state, "closed");
      });
}, 'Decode empty VP9 frame');

promise_test(t => {
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  decoder.configure({codec: vp9.codec});

  let fakeChunk = getFakeChunk();
  decoder.decode(fakeChunk);

  // Create the flush promise before closing, as it is invalid to do so later.
  let flushPromise = decoder.flush();

  // This should synchronously reject the flush() promise.
  decoder.close();

  // TODO(sandersd): Wait for a bit in case there is a lingering output
  // or error coming.
  return promise_rejects_exactly(t, undefined, flushPromise);
}, 'Close while decoding corrupt VP9 frame');
