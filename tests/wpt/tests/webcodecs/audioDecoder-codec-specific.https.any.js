// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: script=audioDecoder-codec-specific-setup.js
// META: variant=?adts_aac
// META: variant=?mp4_aac
// META: variant=?mp3
// META: variant=?opus
// META: variant=?pcm_alaw
// META: variant=?pcm_ulaw
// META: variant=?pcm_u8
// META: variant=?pcm_s16
// META: variant=?pcm_s24
// META: variant=?pcm_s32
// META: variant=?pcm_f32
// META: variant=?flac
// META: variant=?vorbis

promise_test(t => {
  return AudioDecoder.isConfigSupported(CONFIG);
}, 'Test isConfigSupported()');

promise_test(t => {
  // Define a valid config that includes a hypothetical 'futureConfigFeature',
  // which is not yet recognized by the User Agent.
  const validConfig = {
    ...CONFIG,
    futureConfigFeature: 'foo',
  };

  // The UA will evaluate validConfig as being "valid", ignoring the
  // `futureConfigFeature` it  doesn't recognize.
  return AudioDecoder.isConfigSupported(validConfig).then((decoderSupport) => {
    // AudioDecoderSupport must contain the following properites.
    assert_true(decoderSupport.hasOwnProperty('supported'));
    assert_true(decoderSupport.hasOwnProperty('config'));

    // AudioDecoderSupport.config must not contain unrecognized properties.
    assert_false(decoderSupport.config.hasOwnProperty('futureConfigFeature'));

    // AudioDecoderSupport.config must contiain the recognized properties.
    assert_equals(decoderSupport.config.codec, validConfig.codec);
    assert_equals(decoderSupport.config.sampleRate, validConfig.sampleRate);
    assert_equals(
        decoderSupport.config.numberOfChannels, validConfig.numberOfChannels);

    if (validConfig.description) {
      // The description must be copied.
      assert_false(
          decoderSupport.config.description === validConfig.description,
          'description is unique');
      assert_array_equals(
          new Uint8Array(decoderSupport.config.description, 0),
          new Uint8Array(validConfig.description, 0), 'description');
    } else {
      assert_false(
          decoderSupport.config.hasOwnProperty('description'), 'description');
    }
  });
}, 'Test that AudioDecoder.isConfigSupported() returns a parsed configuration');

promise_test(async t => {
  const decoder = createAudioDecoder(t);
  decoder.configure(CONFIG);
  assert_equals(decoder.state, 'configured', 'state');
}, 'Test configure()');

promise_test(t => {
  const decoder = createAudioDecoder(t);
  return testClosedCodec(t, decoder, CONFIG, CHUNKS[0]);
}, 'Verify closed AudioDecoder operations');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    frame.close();
  };

  decoder.configure(CONFIG);
  CHUNKS.forEach(chunk => {
    decoder.decode(chunk);
  });

  await decoder.flush();
  assert_equals(outputs, CHUNKS.length - (CODEC_DATA.packet_delay ?? 0),
                'outputs');
}, 'Test decoding');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

  let outputs = 0;
  callbacks.output = frame => {
    if (outputs === 0) {
      assert_equals(frame.timestamp, -42 + (CODEC_DATA.discard_padding ?? 0));
    }
    outputs++;
    frame.close();
  };

  decoder.configure(CONFIG);
  decoder.decode(new EncodedAudioChunk(
      {type: 'key', timestamp: -42, data: CHUNK_DATA[0]}));
  decoder.decode(new EncodedAudioChunk(
      {type: 'key', timestamp: CHUNKS[0].duration - 42, data: CHUNK_DATA[1]}));

  await decoder.flush();
  assert_equals(outputs, 2 - (CODEC_DATA.packet_delay ?? 0), 'outputs');
}, 'Test decoding a with a negative timestamp');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

  let outputs = 0;
  callbacks.output = frame => {
    if (outputs === 0) {
      assert_equals(frame.timestamp, 42);
    }
    outputs++;
    frame.close();
  };

  decoder.configure(CONFIG);
  decoder.decode(new EncodedAudioChunk(
      {type: 'key', timestamp: 42, data: CHUNK_DATA[0]}));
  decoder.decode(new EncodedAudioChunk(
      {type: 'key', timestamp: CHUNKS[0].duration + 42, data: CHUNK_DATA[1]}));

  await decoder.flush();
  assert_equals(outputs, 2 - (CODEC_DATA.packet_delay ?? 0), 'outputs');
}, 'Test decoding a with a positive timestamp');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    frame.close();
  };

  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);
  decoder.decode(CHUNKS[1]);

  await decoder.flush();
  assert_equals(outputs, 2 - (CODEC_DATA.packet_delay ?? 0), 'outputs');

  // Flushing resets the decoder, so codecs with a packet delay (e.g. Vorbis)
  // need the first packet to be queued again to prime the decoder.
  if (CODEC_DATA.packet_delay > 0) {
    decoder.decode(CHUNKS[0]);
  }
  decoder.decode(CHUNKS[2]);
  await decoder.flush();
  assert_equals(outputs, 3 - (CODEC_DATA.packet_delay ?? 0), 'outputs');
}, 'Test decoding after flush');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

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
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

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
}, 'AudioDecoder decodeQueueSize test');

if (location.search === '?opus') {
  promise_test(async t => {
    const callbacks = {};
    const decoder = createAudioDecoder(t, callbacks);

    let outputSampleRate = 0;
    callbacks.output = frame => {
      outputSampleRate = frame.sampleRate;
      frame.close();
    };

    let config = Object.assign({}, CONFIG);
    config.sampleRate = 44100;

    decoder.configure(config);
    decoder.decode(CHUNKS[0]);
    await decoder.flush();

    // RFC 7845 Section 5.1: Opus audio is decoded at 48000 Hz regardless of
    // configured rate.
    assert_equals(outputSampleRate, 48000);
  }, 'Test Opus decoding with non-48k sample rate');
}
