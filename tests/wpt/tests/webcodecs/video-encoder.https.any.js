// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js

const defaultConfig = {
  codec: 'vp8',
  width: 640,
  height: 480
};

promise_test(t => {
  // VideoEncoderInit lacks required fields.
  assert_throws_js(TypeError, () => { new VideoEncoder({}); });

  // VideoEncoderInit has required fields.
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  assert_equals(encoder.state, "unconfigured");

  encoder.close();

  return endAfterEventLoopTurn();
}, 'Test VideoEncoder construction');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  let encoderConfig = {
    codec: 'vp8',
    width: 640,
    height: 480,
    displayWidth: 800,
    displayHeight: 600,
  };

  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  encoder.configure(encoderConfig);

  let frame1 = createFrame(640, 480, 0);
  let frame2 = createFrame(640, 480, 33333);
  t.add_cleanup(() => {
    frame1.close();
    frame2.close();
  });

  encoder.encode(frame1);
  encoder.encode(frame2);

  await encoder.flush();

  // Decoder config should be given with the first chunk
  assert_not_equals(decoderConfig, null);
  assert_equals(decoderConfig.codec, encoderConfig.codec);
  assert_greater_than_equal(decoderConfig.codedHeight, encoderConfig.height);
  assert_greater_than_equal(decoderConfig.codedWidth, encoderConfig.width);
  assert_equals(decoderConfig.displayAspectHeight, encoderConfig.displayHeight);
  assert_equals(decoderConfig.displayAspectWidth, encoderConfig.displayWidth);
  assert_not_equals(decoderConfig.colorSpace.primaries, null);
  assert_not_equals(decoderConfig.colorSpace.transfer, null);
  assert_not_equals(decoderConfig.colorSpace.matrix, null);
  assert_not_equals(decoderConfig.colorSpace.fullRange, null);

  assert_equals(output_chunks.length, 2);
  assert_equals(output_chunks[0].timestamp, frame1.timestamp);
  assert_equals(output_chunks[0].duration, frame1.duration);
  assert_equals(output_chunks[1].timestamp, frame2.timestamp);
  assert_equals(output_chunks[1].duration, frame2.duration);
}, 'Test successful configure(), encode(), and flush()');

promise_test(async t => {
  let codecInit = getDefaultCodecInit(t);
  let encoderConfig = {
    codec: 'vp8',
    width: 320,
    height: 200
  };

  codecInit.output = (chunk, metadata) => {}

  let encoder = new VideoEncoder(codecInit);

  // No encodes yet.
  assert_equals(encoder.encodeQueueSize, 0);

  encoder.configure(encoderConfig);

  // Still no encodes.
  assert_equals(encoder.encodeQueueSize, 0);

  const frames_count = 100;
  let frames = [];
  for (let i = 0; i < frames_count; i++) {
    let frame = createFrame(320, 200, i * 16000);
    frames.push(frame);
  }

  let lastDequeueSize = Infinity;
  encoder.ondequeue = () => {
    assert_greater_than(lastDequeueSize, 0, "Dequeue event after queue empty");
    assert_greater_than(lastDequeueSize, encoder.encodeQueueSize,
                        "Dequeue event without decreased queue size");
    lastDequeueSize = encoder.encodeQueueSize;
  };

  for (let frame of frames)
    encoder.encode(frame);

  assert_greater_than_equal(encoder.encodeQueueSize, 0);
  assert_less_than_equal(encoder.encodeQueueSize, frames_count);

  await encoder.flush();
  // We can guarantee that all encodes are processed after a flush.
  assert_equals(encoder.encodeQueueSize, 0);
  // Last dequeue event should fire when the queue is empty.
  assert_equals(lastDequeueSize, 0);

  // Reset this to Infinity to track the decline of queue size for this next
  // batch of encodes.
  lastDequeueSize = Infinity;

  for (let frame of frames) {
    encoder.encode(frame);
    frame.close();
  }

  assert_greater_than_equal(encoder.encodeQueueSize, 0);
  encoder.reset();
  assert_equals(encoder.encodeQueueSize, 0);
}, 'encodeQueueSize test');


promise_test(async t => {
  let timestamp = 0;
  let callbacks_before_reset = 0;
  let callbacks_after_reset = 0;
  const timestamp_step = 40000;
  const expected_callbacks_before_reset = 3;
  let codecInit = getDefaultCodecInit(t);
  let original = createFrame(320, 200, 0);
  let encoder = null;
  let reset_completed = false;
  codecInit.output = (chunk, metadata) => {
    if (chunk.timestamp % 2 == 0) {
      // pre-reset frames have even timestamp
      callbacks_before_reset++;
      if (callbacks_before_reset == expected_callbacks_before_reset) {
        encoder.reset();
        reset_completed = true;
      }
    } else {
      // after-reset frames have odd timestamp
      callbacks_after_reset++;
    }
  }

  encoder = new VideoEncoder(codecInit);
  encoder.configure(defaultConfig);
  await encoder.flush();

  // Send 10x frames to the encoder, call reset() on it after x outputs,
  // and make sure no more chunks are emitted afterwards.
  let encodes_before_reset = expected_callbacks_before_reset * 10;
  for (let i = 0; i < encodes_before_reset; i++) {
    let frame = new VideoFrame(original, { timestamp: timestamp });
    timestamp += timestamp_step;
    encoder.encode(frame);
    frame.close();
  }

  await t.step_wait(() => reset_completed,
    "Reset() should be called by output callback", 10000, 1);

  assert_equals(callbacks_before_reset, expected_callbacks_before_reset);
  assert_true(reset_completed);
  assert_equals(encoder.encodeQueueSize, 0);

  let newConfig = { ...defaultConfig };
  newConfig.width = 800;
  newConfig.height = 600;
  encoder.configure(newConfig);

  const frames_after_reset = 5;
  for (let i = 0; i < frames_after_reset; i++) {
    let frame = createFrame(800, 600, timestamp + 1);
    timestamp += timestamp_step;
    encoder.encode(frame);
    frame.close();
  }
  await encoder.flush();

  assert_equals(callbacks_after_reset, frames_after_reset,
    "not all after-reset() outputs have been emitted");
  assert_equals(callbacks_before_reset, expected_callbacks_before_reset,
    "pre-reset() outputs were emitter after reset() and flush()");
  assert_equals(encoder.encodeQueueSize, 0);
}, 'Test successful reset() and re-confiugre()');

promise_test(async t => {
  let output_chunks = [];
  const codecInit = {
    output: chunk => output_chunks.push(chunk),
  };
  const error = new Promise(resolve => codecInit.error = e => {
    resolve(e);
  });

  let encoder = new VideoEncoder(codecInit);

  // No encodes yet.
  assert_equals(encoder.encodeQueueSize, 0);

  let config = defaultConfig;

  encoder.configure(config);

  let frame1 = createFrame(640, 480, 0);
  let frame2 = createFrame(640, 480, 33333);

  encoder.encode(frame1);
  encoder.configure(config);

  encoder.encode(frame2);

  await encoder.flush();

  // We can guarantee that all encodes are processed after a flush.
  assert_equals(encoder.encodeQueueSize, 0, "queue size after encode");

  assert_equals(output_chunks.length, 2, "number of chunks");
  assert_equals(output_chunks[0].timestamp, frame1.timestamp);
  assert_equals(output_chunks[1].timestamp, frame2.timestamp);

  output_chunks = [];

  let frame3 = createFrame(640, 480, 66666);

  encoder.encode(frame3);

  let badConfig = { ...defaultConfig };
  badConfig.codec = '';
  assert_throws_js(TypeError, () => encoder.configure(badConfig));

  badConfig.codec = 'bogus';
  encoder.configure(badConfig);
  let e = await error;
  assert_true(e instanceof DOMException);
  assert_equals(e.name, 'NotSupportedError');
  assert_equals(encoder.state, 'closed', 'state');

  // We may or may not have received frame3 before closing.
}, 'Test successful encode() after re-configure().');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = createFrame(640, 480, 0);

  return testClosedCodec(t, encoder, defaultConfig, frame);
}, 'Verify closed VideoEncoder operations');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = createFrame(640, 480, 0);

  return testUnconfiguredCodec(t, encoder, frame);
}, 'Verify unconfigured VideoEncoder operations');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = createFrame(640, 480, 0);
  frame.close();

  encoder.configure(defaultConfig);

  assert_throws_js(TypeError, () => {
    encoder.encode(frame);
  });
}, 'Verify encoding closed frames throws.');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  codecInit.output = chunk => output_chunks.push(chunk);

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame = createFrame(640, 480, -10000);
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  encoder.close();
  assert_equals(output_chunks.length, 1);
  assert_equals(output_chunks[0].timestamp, -10000, "first chunk timestamp");
  assert_greater_than(output_chunks[0].byteLength, 0);
}, 'Encode video with negative timestamp');
