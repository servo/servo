// META: global=window,dedicatedworker
// META: script=videoDecoder-codec-specific-setup.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9
// META: variant=?h264_avc
// META: variant=?h264_annexb
// META: variant=?h265_hevc
// META: variant=?h265_annexb

promise_test(async t => {
  await checkImplements();
  const support = await VideoDecoder.isConfigSupported(CONFIG);
  assert_true(support.supported, 'supported');
}, 'Test isConfigSupported()');

promise_test(async t => {
  await checkImplements();
  // TODO(sandersd): Create a 1080p `description` for H.264 in AVC format.
  // This version is testing only the H.264 Annex B path.
  const config = {
    codec: CONFIG.codec,
    codedWidth: 1920,
    codedHeight: 1088,
    displayAspectWidth: 1920,
    displayAspectHeight: 1080,
  };

  const support = await VideoDecoder.isConfigSupported(config);
  assert_true(support.supported, 'supported');
}, 'Test isConfigSupported() with 1080p crop');

promise_test(async t => {
  await checkImplements();
  // Define a valid config that includes a hypothetical `futureConfigFeature`,
  // which is not yet recognized by the User Agent.
  const config = {
    ...CONFIG,
    colorSpace: {primaries: 'bt709'},
    futureConfigFeature: 'foo',
  };

  // The UA will evaluate validConfig as being "valid", ignoring the
  // `futureConfigFeature` it  doesn't recognize.
  const support = await VideoDecoder.isConfigSupported(config);
  assert_true(support.supported, 'supported');
  assert_equals(support.config.codec, config.codec, 'codec');
  assert_equals(support.config.codedWidth, config.codedWidth, 'codedWidth');
  assert_equals(support.config.codedHeight, config.codedHeight, 'codedHeight');
  assert_equals(support.config.displayAspectWidth, config.displayAspectWidth, 'displayAspectWidth');
  assert_equals(support.config.displayAspectHeight, config.displayAspectHeight, 'displayAspectHeight');
  assert_equals(support.config.colorSpace.primaries, config.colorSpace.primaries, 'color primaries');
  assert_equals(support.config.colorSpace.transfer, null, 'color transfer');
  assert_equals(support.config.colorSpace.matrix, null, 'color matrix');
  assert_equals(support.config.colorSpace.fullRange, null, 'color range');
  assert_false(support.config.hasOwnProperty('futureConfigFeature'), 'futureConfigFeature');

  if (config.description) {
    // The description must be copied.
    assert_false(
        support.config.description === config.description,
        'description is unique');
    assert_array_equals(
        new Uint8Array(support.config.description, 0),
        new Uint8Array(config.description, 0), 'description');
  } else {
    assert_false(support.config.hasOwnProperty('description'), 'description');
  }
}, 'Test that isConfigSupported() returns a parsed configuration');

promise_test(async t => {
  await checkImplements();
  async function test(t, config, description) {
    await promise_rejects_js(
        t, TypeError, VideoDecoder.isConfigSupported(config), description);

    const decoder = createVideoDecoder(t);
    assert_throws_js(TypeError, () => decoder.configure(config), description);
    assert_equals(decoder.state, 'unconfigured', 'state');
  }

  await test(t, {...CONFIG, codedWidth: 0}, 'invalid codedWidth');
  await test(t, {...CONFIG, displayAspectWidth: 0}, 'invalid displayAspectWidth');
}, 'Test invalid configs');

promise_test(async t => {
  await checkImplements();
  const decoder = createVideoDecoder(t);
  decoder.configure(CONFIG);
  assert_equals(decoder.state, 'configured', 'state');
}, 'Test configure()');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, CHUNKS[0].timestamp, 'timestamp');
    assert_equals(frame.duration, CHUNKS[0].duration, 'duration');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'Decode a key frame');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);

  // Ensure type value is verified.
  assert_equals(CHUNKS[1].type, 'delta');
  assert_throws_dom('DataError', () => decoder.decode(CHUNKS[1], 'decode'));
}, 'Decode a non key frame first fails');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  for (let i = 0; i < 16; i++) {
    decoder.decode(new EncodedVideoChunk(
        {type: 'key', timestamp: 0, data: CHUNK_DATA[0]}));
  }
  assert_greater_than(decoder.decodeQueueSize, 0);

  // Wait for the first output, then reset the decoder.
  let outputs = 0;
  await new Promise(resolve => {
    callbacks.output = frame => {
      outputs++;
      assert_equals(outputs, 1, 'outputs');
      assert_equals(frame.timestamp, 0, 'timestamp');
      frame.close();
      decoder.reset();
      assert_equals(decoder.decodeQueueSize, 0, 'decodeQueueSize');
      resolve();
    };
  });

  decoder.configure(CONFIG);
  for (let i = 0; i < 4; i++) {
    decoder.decode(new EncodedVideoChunk(
        {type: 'key', timestamp: 1, data: CHUNK_DATA[0]}));
  }

  // Expect future outputs to come from after the reset.
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, 1, 'timestamp');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 5);
  assert_equals(decoder.decodeQueueSize, 0);
}, 'Verify reset() suppresses outputs');

promise_test(async t => {
  await checkImplements();
  const decoder = createVideoDecoder(t);
  assert_equals(decoder.state, 'unconfigured');

  decoder.reset();
  assert_equals(decoder.state, 'unconfigured');
  assert_throws_dom(
      'InvalidStateError', () => decoder.decode(CHUNKS[0]), 'decode');
  await promise_rejects_dom(t, 'InvalidStateError', decoder.flush(), 'flush');
}, 'Test unconfigured VideoDecoder operations');

promise_test(async t => {
  await checkImplements();
  const decoder = createVideoDecoder(t);
  decoder.close();
  assert_equals(decoder.state, 'closed');
  assert_throws_dom(
      'InvalidStateError', () => decoder.configure(CONFIG), 'configure');
  assert_throws_dom('InvalidStateError', () => decoder.reset(), 'reset');
  assert_throws_dom('InvalidStateError', () => decoder.close(), 'close');
  assert_throws_dom(
      'InvalidStateError', () => decoder.decode(CHUNKS[0]), 'decode');
  await promise_rejects_dom(t, 'InvalidStateError', decoder.flush(), 'flush');
}, 'Test closed VideoDecoder operations');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};

  let errors = 0;
  let gotError = new Promise(resolve => callbacks.error = e => {
    errors++;
    resolve(e);
  });
  callbacks.output = frame => { frame.close(); };

  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);  // Decode keyframe first.
  decoder.decode(new EncodedVideoChunk(
      {type: 'key', timestamp: 1, data: new ArrayBuffer(0)}));

  await promise_rejects_dom(t, "EncodingError",
    decoder.flush().catch((e) => {
      assert_equals(errors, 1);
      throw e;
    })
  );

  let e = await gotError;
  assert_true(e instanceof DOMException);
  assert_equals(e.name, 'EncodingError');
  assert_equals(decoder.state, 'closed', 'state');
}, 'Decode empty frame');


promise_test(async t => {
  await checkImplements();
  const callbacks = {};

  let errors = 0;
  let gotError = new Promise(resolve => callbacks.error = e => {
    errors++;
    resolve(e);
  });

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    frame.close();
  };

  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);  // Decode keyframe first.
  decoder.decode(createCorruptChunk(2));

  await promise_rejects_dom(t, "EncodingError",
    decoder.flush().catch((e) => {
      assert_equals(errors, 1);
      throw e;
    })
  );

  assert_less_than_equal(outputs, 1);
  let e = await gotError;
  assert_true(e instanceof DOMException);
  assert_equals(e.name, 'EncodingError');
  assert_equals(decoder.state, 'closed', 'state');
}, 'Decode corrupt frame');

promise_test(async t => {
  await checkImplements();
  const decoder = createVideoDecoder(t);

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);  // Decode keyframe first.
  decoder.decode(createCorruptChunk(1));

  let flushDone = decoder.flush();
  decoder.close();

  // Flush should have been synchronously rejected, with no output() or error()
  // callbacks.
  await promise_rejects_dom(t, 'AbortError', flushDone);
}, 'Close while decoding corrupt frame');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');

  decoder.decode(CHUNKS[0]);
  await decoder.flush();
  assert_equals(outputs, 2, 'outputs');
}, 'Test decoding after flush');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure(CONFIG);
  decoder.decode(new EncodedVideoChunk(
      {type: 'key', timestamp: -42, data: CHUNK_DATA[0]}));

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, -42, 'timestamp');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'Test decoding a with negative timestamp');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);
  decoder.decode(CHUNKS[1]);
  const flushDone = decoder.flush();

  // Wait for the first output, then reset.
  let outputs = 0;
  await new Promise(resolve => {
    callbacks.output = frame => {
      outputs++;
      assert_equals(outputs, 1, 'outputs');
      decoder.reset();
      frame.close();
      resolve();
    };
  });

  // Flush should have been synchronously rejected.
  await promise_rejects_dom(t, 'AbortError', flushDone);

  assert_equals(outputs, 1, 'outputs');
}, 'Test reset during flush');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);
  const flushDone = decoder.flush();

  let flushDoneInCallback;
  let outputs = 0;
  await new Promise(resolve => {
    callbacks.output = frame => {
      decoder.reset();
      frame.close();

      callbacks.output = frame => {
        outputs++;
        frame.close();
      };
      callbacks.error = e => {
        t.unreached_func('unexpected error()');
      };
      decoder.configure(CONFIG);
      decoder.decode(CHUNKS[0]);
      flushDoneInCallback = decoder.flush();

      resolve();
    };
  });

  // First flush should have been synchronously rejected.
  await promise_rejects_dom(t, 'AbortError', flushDone);
  // Wait for the second flush and check the output count.
  await flushDoneInCallback;
  assert_equals(outputs, 1, 'outputs');
}, 'Test new flush after reset in a flush callback');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);
  const flushDone = decoder.flush();
  let flushDoneInCallback;

  await new Promise(resolve => {
    callbacks.output = frame => {
      decoder.reset();
      frame.close();

      callbacks.output = frame => { frame.close(); };
      decoder.configure(CONFIG);
      decoder.decode(CHUNKS[0]);
      decoder.decode(createCorruptChunk(1));
      flushDoneInCallback = decoder.flush();

      resolve();
    };
  });

  // First flush should have been synchronously rejected.
  await promise_rejects_dom(t, 'AbortError', flushDone);
  // Wait for the second flush and check the error in the rejected promise.
  await promise_rejects_dom(t, 'EncodingError', flushDoneInCallback);
}, 'Test decoding a corrupt frame after reset in a flush callback');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);

  decoder.configure({...CONFIG, optimizeForLatency: true});
  decoder.decode(CHUNKS[0]);

  // The frame should be output without flushing.
  await new Promise(resolve => {
    callbacks.output = frame => {
      frame.close();
      resolve();
    };
  });
}, 'Test low-latency decoding');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  callbacks.output = frame => { frame.close(); };
  const decoder = createVideoDecoder(t, callbacks);

  // No decodes yet.
  assert_equals(decoder.decodeQueueSize, 0);

  decoder.configure(CONFIG);

  // Still no decodes.
  assert_equals(decoder.decodeQueueSize, 0);

  let lastDequeueSize = Infinity;
  decoder.ondequeue = () => {
    assert_greater_than(lastDequeueSize, 0, "Dequeue event after queue empty");
    assert_greater_than(lastDequeueSize, decoder.decodeQueueSize,
                        "Dequeue event without decreased queue size");
    lastDequeueSize = decoder.decodeQueueSize;
  };

  for (let chunk of CHUNKS)
    decoder.decode(chunk);

  assert_greater_than_equal(decoder.decodeQueueSize, 0);
  assert_less_than_equal(decoder.decodeQueueSize, CHUNKS.length);

  await decoder.flush();
  // We can guarantee that all decodes are processed after a flush.
  assert_equals(decoder.decodeQueueSize, 0);
  // Last dequeue event should fire when the queue is empty.
  assert_equals(lastDequeueSize, 0);

  // Reset this to Infinity to track the decline of queue size for this next
  // batch of decodes.
  lastDequeueSize = Infinity;

  for (let chunk of CHUNKS)
    decoder.decode(chunk);

  assert_greater_than_equal(decoder.decodeQueueSize, 0);
  decoder.reset();
  assert_equals(decoder.decodeQueueSize, 0);
}, 'VideoDecoder decodeQueueSize test');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  decoder.reset();
  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, CHUNKS[0].timestamp, 'timestamp');
    assert_equals(frame.duration, CHUNKS[0].duration, 'duration');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'Test configure, reset, configure does not stall');
