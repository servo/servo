// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

const defaultConfig = {
  codec: 'vp8',
  framerate: 25,
  width: 640,
  height: 480
};

async function generateBitmap(width, height) {
  const src = "pattern.png";

  var size = {
    resizeWidth: width,
    resizeHeight: height
  };

  return fetch(src)
      .then(response => response.blob())
      .then(blob => createImageBitmap(blob, size));
}

async function createVideoFrame(width, height, timestamp) {
  let bitmap = await generateBitmap(width, height);
  return new VideoFrame(bitmap, { timestamp: timestamp });
}

promise_test(t => {
  // VideoEncoderInit lacks required fields.
  assert_throws_js(TypeError, () => { new VideoEncoder({}); });

  // VideoEncoderInit has required fields.
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  assert_equals(encoder.state, "unconfigured");

  encoder.close();

  return endAfterEventLoopTurn();
}, 'Test VideoEncoder construction');

promise_test(t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let badCodecsList = [
    '',                         // Empty codec
    'bogus',                    // Non exsitent codec
    'vorbis',                   // Audio codec
    'vp9',                      // Ambiguous codec
    'video/webm; codecs="vp9"'  // Codec with mime type
  ]

  testConfigurations(encoder, defaultConfig, badCodecsList);

  return endAfterEventLoopTurn();
}, 'Test VideoEncoder.configure()');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  codecInit.output = chunk => output_chunks.push(chunk);

  let encoder = new VideoEncoder(codecInit);

  // No encodes yet.
  assert_equals(encoder.encodeQueueSize, 0);

  encoder.configure(defaultConfig);

  // Still no encodes.
  assert_equals(encoder.encodeQueueSize, 0);

  let frame1 = await createVideoFrame(640, 480, 0);
  let frame2 = await createVideoFrame(640, 480, 33333);

  encoder.encode(frame1.clone());
  encoder.encode(frame2.clone());

  // Could be 0, 1, or 2. We can't guarantee this check runs before the UA has
  // processed the encodes.
  assert_true(encoder.encodeQueueSize >= 0 && encoder.encodeQueueSize <= 2)

  await encoder.flush();

  // We can guarantee that all encodes are processed after a flush.
  assert_equals(encoder.encodeQueueSize, 0);

  assert_equals(output_chunks.length, 2);
  assert_equals(output_chunks[0].timestamp, frame1.timestamp);
  assert_equals(output_chunks[1].timestamp, frame2.timestamp);
}, 'Test successful configure(), encode(), and flush()');

promise_test(async t => {
  let callbacks_before_reset = 0;
  let callbacks_after_reset = 0;
  let codecInit = getDefaultCodecInit(t);
  codecInit.output = chunk => {
    if (chunk.timestamp % 2 == 0) {
      // pre-reset frames have even timestamp
      callbacks_before_reset++;
    } else {
      // after-reset frames have odd timestamp
      callbacks_after_reset++;
    }
  }

  let encoder = new VideoEncoder(codecInit);
  encoder.configure(defaultConfig);
  await encoder.flush();

  let frames = [];
  for (let i = 0; i < 200; i++) {
    let frame = await createVideoFrame(640, 480, i * 40_000);
    frames.push(frame);
  }

  for (frame of frames)
    encoder.encode(frame);

  // Wait for the first frame to be encoded
  await t.step_wait(() => callbacks_before_reset > 0,
    "Encoded outputs started coming", 10000, 1);

  let saved_callbacks_before_reset = callbacks_before_reset;
  assert_greater_than(callbacks_before_reset, 0);
  assert_less_than_equal(callbacks_before_reset, frames.length);

  encoder.reset();
  assert_equals(encoder.encodeQueueSize, 0);

  let newConfig = { ...defaultConfig };
  newConfig.width = 800;
  newConfig.height = 600;
  encoder.configure(newConfig);

  for (let i = frames.length; i < frames.length + 5; i++) {
    let frame = await createVideoFrame(800, 600, i * 40_000 + 1);
    encoder.encode(frame);
  }
  await encoder.flush();
  assert_equals(callbacks_after_reset, 5);
  assert_equals(saved_callbacks_before_reset, callbacks_before_reset);
  assert_equals(encoder.encodeQueueSize, 0);
}, 'Test successful reset() and re-confiugre()');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  codecInit.output = chunk => output_chunks.push(chunk);

  let encoder = new VideoEncoder(codecInit);

  // No encodes yet.
  assert_equals(encoder.encodeQueueSize, 0);

  let config = defaultConfig;

  encoder.configure(config);

  let frame1 = await createVideoFrame(640, 480, 0);
  let frame2 = await createVideoFrame(640, 480, 33333);

  encoder.encode(frame1.clone());
  encoder.configure(config);

  encoder.encode(frame2.clone());

  await encoder.flush();

  // We can guarantee that all encodes are processed after a flush.
  assert_equals(encoder.encodeQueueSize, 0);

  // The first frame may have been dropped when reconfiguring.
  // This shouldn't happen, and should be fixed/called out in the spec, but
  // this is preptively added to prevent flakiness.
  // TODO: Remove these checks when implementations handle this correctly.
  assert_true(output_chunks.length == 1 || output_chunks.length == 2);

  if (output_chunks.length == 1) {
    // If we only have one chunk frame, make sure we droped the frame that was
    // in flight when we reconfigured.
    assert_equals(output_chunks[0].timestamp, frame2.timestamp);
  } else {
    assert_equals(output_chunks[0].timestamp, frame1.timestamp);
    assert_equals(output_chunks[1].timestamp, frame2.timestamp);
  }

  output_chunks = [];

  let frame3 = await createVideoFrame(640, 480, 66666);
  let frame4 = await createVideoFrame(640, 480, 100000);

  encoder.encode(frame3.clone());

  // Verify that a failed call to configure does not change the encoder's state.
  let badConfig = { ...defaultConfig };
  badConfig.codec = 'bogus';
  assert_throws_js(TypeError, () => encoder.configure(badConfig));

  encoder.encode(frame4.clone());

  await encoder.flush();

  assert_equals(output_chunks[0].timestamp, frame3.timestamp);
  assert_equals(output_chunks[1].timestamp, frame4.timestamp);
}, 'Test successful encode() after re-configure().');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  codecInit.output = chunk => output_chunks.push(chunk);

  let encoder = new VideoEncoder(codecInit);

  let timestamp = 33333;
  let frame = await createVideoFrame(640, 480, timestamp);

  encoder.configure(defaultConfig);
  assert_equals(encoder.state, "configured");

  encoder.encode(frame);

  // |frame| is not longer valid since it has been destroyed.
  assert_not_equals(frame.timestamp, timestamp);
  assert_throws_dom("InvalidStateError", () => frame.clone());

  encoder.close();

  return endAfterEventLoopTurn();
}, 'Test encoder consumes (destroys) frames.');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = await createVideoFrame(640, 480, 0);

  return testClosedCodec(t, encoder, defaultConfig, frame);
}, 'Verify closed VideoEncoder operations');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = await createVideoFrame(640, 480, 0);

  return testUnconfiguredCodec(t, encoder, frame);
}, 'Verify unconfigured VideoEncoder operations');

promise_test(async t => {
  let encoder = new VideoEncoder(getDefaultCodecInit(t));

  let frame = await createVideoFrame(640, 480, 0);
  frame.destroy();

  encoder.configure(defaultConfig);

  frame.destroy();
  assert_throws_dom("OperationError", () => {
    encoder.encode(frame)
  });
}, 'Verify encoding destroyed frames throws.');
