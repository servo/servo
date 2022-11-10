// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: variant=?adts_aac
// META: variant=?mp4_aac
// META: variant=?mp3
// META: variant=?opus
// META: variant=?pcm_alaw
// META: variant=?pcm_mulaw

const ADTS_AAC_DATA = {
  src: 'sfx.adts',
  config: {
    codec: 'mp4a.40.2',
    sampleRate: 48000,
    numberOfChannels: 1,
  },
  chunks: [
    {offset: 0, size: 248}, {offset: 248, size: 280}, {offset: 528, size: 258},
    {offset: 786, size: 125}, {offset: 911, size: 230},
    {offset: 1141, size: 148}, {offset: 1289, size: 224},
    {offset: 1513, size: 166}, {offset: 1679, size: 216},
    {offset: 1895, size: 183}
  ],
  duration: 24000
};

const MP3_DATA = {
  src: 'sfx.mp3',
  config: {
    codec: 'mp3',
    sampleRate: 48000,
    numberOfChannels: 1,
  },
  chunks: [
    {offset: 333, size: 288}, {offset: 621, size: 288},
    {offset: 909, size: 288}, {offset: 1197, size: 288},
    {offset: 1485, size: 288}, {offset: 1773, size: 288},
    {offset: 2061, size: 288}, {offset: 2349, size: 288},
    {offset: 2637, size: 288}, {offset: 2925, size: 288}
  ],
  duration: 24000
};

const MP4_AAC_DATA = {
  src: 'sfx-aac.mp4',
  config: {
    codec: 'mp4a.40.2',
    sampleRate: 48000,
    numberOfChannels: 1,
    description: {offset: 2552, size: 5},
  },
  chunks: [
    {offset: 44, size: 241},
    {offset: 285, size: 273},
    {offset: 558, size: 251},
    {offset: 809, size: 118},
    {offset: 927, size: 223},
    {offset: 1150, size: 141},
    {offset: 1291, size: 217},
    {offset: 1508, size: 159},
    {offset: 1667, size: 209},
    {offset: 1876, size: 176},
  ],
  duration: 21333
};

const OPUS_DATA = {
  src: 'sfx-opus.ogg',
  config: {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 1,
    description: {offset: 28, size: 19},
  },
  chunks: [
    {offset: 185, size: 450}, {offset: 635, size: 268},
    {offset: 903, size: 285}, {offset: 1188, size: 296},
    {offset: 1484, size: 287}, {offset: 1771, size: 308},
    {offset: 2079, size: 289}, {offset: 2368, size: 286},
    {offset: 2654, size: 296}, {offset: 2950, size: 294}
  ],
  duration: 20000
};

const PCM_ALAW_DATA = {
  src: 'sfx-alaw.wav',
  config: {
    codec: 'alaw',
    sampleRate: 48000,
    numberOfChannels: 1,
  },
  // Any arbitrary grouping should work.
  chunks: [
    {offset: 0, size: 2048}, {offset: 2048, size: 2048},
    {offset: 4096, size: 2048}, {offset: 6144, size: 2048},
    {offset: 8192, size: 2048}, {offset: 10240, size: 92}
  ],
  duration: 35555
};

const PCM_MULAW_DATA = {
  src: 'sfx-mulaw.wav',
  config: {
    codec: 'ulaw',
    sampleRate: 48000,
    numberOfChannels: 1,
  },

  // Any arbitrary grouping should work.
  chunks: [
    {offset: 0, size: 2048}, {offset: 2048, size: 2048},
    {offset: 4096, size: 2048}, {offset: 6144, size: 2048},
    {offset: 8192, size: 2048}, {offset: 10240, size: 92}
  ],
  duration: 35555
};

// Allows mutating `callbacks` after constructing the AudioDecoder, wraps calls
// in t.step().
function createAudioDecoder(t, callbacks) {
  return new AudioDecoder({
    output(frame) {
      if (callbacks && callbacks.output) {
        t.step(() => callbacks.output(frame));
      } else {
        t.unreached_func('unexpected output()');
      }
    },
    error(e) {
      if (callbacks && callbacks.error) {
        t.step(() => callbacks.error(e));
      } else {
        t.unreached_func('unexpected error()');
      }
    }
  });
}

// Create a view of an ArrayBuffer.
function view(buffer, {offset, size}) {
  return new Uint8Array(buffer, offset, size);
}

let CONFIG = null;
let CHUNK_DATA = null;
let CHUNKS = null;
promise_setup(async () => {
  const data = {
    '?adts_aac': ADTS_AAC_DATA,
    '?mp3': MP3_DATA,
    '?mp4_aac': MP4_AAC_DATA,
    '?opus': OPUS_DATA,
    '?pcm_alaw': PCM_ALAW_DATA,
    '?pcm_mulaw': PCM_MULAW_DATA,
  }[location.search];

  // Don't run any tests if the codec is not supported.
  let supported = false;
  try {
    const support = await AudioDecoder.isConfigSupported({
      codec: data.config.codec,
      sampleRate: data.config.sampleRate,
      numberOfChannels: data.config.numberOfChannels
    });
    supported = support.supported;
  } catch (e) {
  }
  assert_implements_optional(supported, data.config.codec + ' unsupported');

  // Fetch the media data and prepare buffers.
  const response = await fetch(data.src);
  const buf = await response.arrayBuffer();

  CONFIG = {...data.config};
  if (data.config.description) {
    CONFIG.description = view(buf, data.config.description);
  }

  CHUNK_DATA = data.chunks.map((chunk, i) => view(buf, chunk));

  CHUNKS = CHUNK_DATA.map((encodedData, i) => new EncodedAudioChunk({
                            type: 'key',
                            timestamp: i * data.duration,
                            duration: data.duration,
                            data: encodedData
                          }));
});

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
  assert_equals(outputs, CHUNKS.length, 'outputs');
}, 'Test decoding');

promise_test(async t => {
  const callbacks = {};
  const decoder = createAudioDecoder(t, callbacks);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    frame.close();
  };

  decoder.configure(CONFIG);
  decoder.decode(new EncodedAudioChunk(
      {type: 'key', timestamp: -42, data: CHUNK_DATA[0]}));

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'Test decoding a with negative timestamp');

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

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');

  decoder.decode(CHUNKS[0]);
  await decoder.flush();
  assert_equals(outputs, 2, 'outputs');
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
