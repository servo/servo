// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js

const defaultConfig = {
  codec: 'vp8',
  width: 640,
  height: 480
};

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame = createFrame(640, 480, 0, {rotation: 90, flip: true});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  encoder.close();
  assert_equals(output_chunks.length, 1);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);
}, 'Encode video frame with orientation');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame1 = createFrame(640, 480, 0, {rotation: 90, flip: true});
  let frame2 = createFrame(640, 480, 33333, {rotation: 90, flip: false});
  let frame3 = createFrame(640, 480, 66666, {rotation: 180, flip: true});
  let frame4 = createFrame(640, 480, 99999, {rotation: 90, flip: true});

  encoder.encode(frame1);
  assert_throws_dom('DataError', () => encoder.encode(frame2));
  assert_throws_dom('DataError', () => encoder.encode(frame3));
  encoder.encode(frame4);

  frame1.close();
  frame2.close();
  frame3.close();
  frame4.close();

  await encoder.flush();
  encoder.close();
  assert_equals(output_chunks.length, 2);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);
}, 'Encode video frames with different orientation has non-fatal failures');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame = createFrame(640, 480, 0, {rotation: 90, flip: true});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  assert_equals(output_chunks.length, 1);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);

  encoder.configure(config);
  frame = createFrame(640, 480, 0, {rotation: 270, flip: false});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  assert_equals(output_chunks.length, 2);
  assert_equals(decoderConfig.rotation, 270);
  assert_equals(decoderConfig.flip, false);

  encoder.close();
}, 'Encode video frames with different orientations after reconfigure');

const kWidth = 64;
const kHeight = 64;
const kConfig = {
  codec: 'vp8',
  width: kWidth,
  height: kHeight,
  bitrate: 1000000,
  framerate: 30,
};

// Creates a configured encoder collecting its chunks and metadata.
async function createEncoder(t, config = kConfig) {
  const support = await VideoEncoder.isConfigSupported(config);
  assert_implements_optional(support.supported, 'VP8 encoding is not supported');
  const outputs = [];
  const encoder = new VideoEncoder({
    output: (chunk, metadata) => outputs.push({chunk, metadata}),
    error: e => assert_unreached(`encoder error: ${e}`),
  });
  t.add_cleanup(() => {
    if (encoder.state !== 'closed')
      encoder.close();
  });
  encoder.configure(config);
  return {encoder, outputs};
}

function encodeFrame(encoder, timestamp, orientation, keyFrame = timestamp === 0) {
  const frame = createFrame(kWidth, kHeight, timestamp, orientation);
  try {
    encoder.encode(frame, {keyFrame});
  } finally {
    frame.close();
  }
}

function firstDecoderConfig(outputs) {
  const output = outputs.find(({metadata}) => metadata?.decoderConfig);
  assert_true(!!output, 'an output has a decoderConfig');
  return output.metadata.decoderConfig;
}

promise_test(async t => {
  const {encoder, outputs} = await createEncoder(t);
  encodeFrame(encoder, 0);
  await encoder.flush();
  const decoderConfig = firstDecoderConfig(outputs);
  assert_equals(decoderConfig.rotation, 0, 'rotation');
  assert_equals(decoderConfig.flip, false, 'flip');
}, 'The decoderConfig of an encoder fed with frames without orientation has no rotation and no flip');

for (const rotation of [0, 90, 180, 270]) {
  for (const flip of [false, true]) {
    promise_test(async t => {
      const {encoder, outputs} = await createEncoder(t);
      encodeFrame(encoder, 0, {rotation, flip});
      encodeFrame(encoder, 33333, {rotation, flip});
      await encoder.flush();
      assert_equals(outputs.length, 2, 'outputs');
      const decoderConfig = firstDecoderConfig(outputs);
      assert_equals(decoderConfig.rotation, rotation, 'rotation');
      assert_equals(decoderConfig.flip, flip, 'flip');
    }, `The decoderConfig of an encoder fed with frames with a rotation of ${rotation} and flip ${flip} has the rotation and flip`);
  }
}

promise_test(async t => {
  const {encoder, outputs} = await createEncoder(t);
  // Rotations are normalized by the frames.
  encodeFrame(encoder, 0, {rotation: 450, flip: true});
  await encoder.flush();
  const decoderConfig = firstDecoderConfig(outputs);
  assert_equals(decoderConfig.rotation, 90, 'rotation');
  assert_equals(decoderConfig.flip, true, 'flip');
}, 'The decoderConfig has the normalized rotation of the frames');

promise_test(async t => {
  const {encoder} = await createEncoder(t);
  encodeFrame(encoder, 0, {rotation: 90, flip: true});
  assert_throws_dom('DataError', () => encodeFrame(encoder, 33333));
}, 'Encoding a frame without orientation after a frame with an orientation throws a DataError');

promise_test(async t => {
  const {encoder} = await createEncoder(t);
  encodeFrame(encoder, 0);
  assert_throws_dom('DataError', () => encodeFrame(encoder, 33333, {rotation: 90}));
}, 'Encoding a rotated frame after a frame without orientation throws a DataError');

promise_test(async t => {
  const {encoder, outputs} = await createEncoder(t);
  encodeFrame(encoder, 0, {rotation: 90, flip: true});
  assert_throws_dom('DataError', () => encodeFrame(encoder, 33333, {rotation: 270, flip: false}));
  // Matching frames are still accepted.
  encodeFrame(encoder, 66666, {rotation: 90, flip: true});
  await encoder.flush();
  assert_equals(outputs.length, 2, 'outputs');
}, 'A frame rejected for its orientation does not prevent matching frames from being encoded');

function decoderConfigs(outputs) {
  return outputs.filter(({metadata}) => metadata?.decoderConfig).map(({metadata}) => metadata.decoderConfig);
}

promise_test(async t => {
  const {encoder, outputs} = await createEncoder(t);
  encodeFrame(encoder, 0, {rotation: 90, flip: false});
  await encoder.flush();
  assert_equals(firstDecoderConfig(outputs).rotation, 90, 'first rotation');

  // The orientation of the next configuration is given by its first frame.
  outputs.length = 0;
  encoder.configure({...kConfig, bitrate: 2000000});
  encodeFrame(encoder, 33333, {rotation: 270, flip: true}, true);
  await encoder.flush();
  assert_equals(outputs.length, 1, 'outputs');
}, 'configure() resets the orientation expected by the encoder');

// A configuration changing only the bitrate and framerate keeps the same internal encoder, while any other change creates a new one once the previous one is flushed.
const kReconfigurations = [
  {description: 'changes only the bitrate', config: {...kConfig, bitrate: 2000000, framerate: 15}},
  {description: 'changes more than the bitrate', config: {...kConfig, width: kWidth / 2, height: kHeight / 2}},
];

for (const {description, config} of kReconfigurations) {
  promise_test(async t => {
    const {encoder, outputs} = await createEncoder(t);
    encodeFrame(encoder, 0, {rotation: 90, flip: false});
    encodeFrame(encoder, 33333, {rotation: 90, flip: false});

    // The orientation of the next frames can change.
    encoder.configure(config);
    encodeFrame(encoder, 66666, {rotation: 270, flip: true}, true);
    encodeFrame(encoder, 100000, {rotation: 270, flip: true});
    await encoder.flush();
    assert_equals(outputs.length, 4, 'outputs');

    // A new decoder config is provided when the orientation of the output changes.
    const configs = decoderConfigs(outputs);
    assert_equals(configs.length, 2, 'decoder configs');
    assert_equals(configs[0].rotation, 90, 'first rotation');
    assert_equals(configs[0].flip, false, 'first flip');
    assert_equals(configs[1].rotation, 270, 'second rotation');
    assert_equals(configs[1].flip, true, 'second flip');
    assert_equals(outputs[2].metadata.decoderConfig, configs[1], 'the decoder config is given with the first output of the new orientation');
  }, `A new decoderConfig is given when the orientation of the encoded frames changes after a configure() that ${description}`);

  promise_test(async t => {
    const {encoder, outputs} = await createEncoder(t);
    // The outputs of the first frame are produced after configure() is called, they describe their frame.
    encodeFrame(encoder, 0, {rotation: 90, flip: true});
    encoder.configure(config);
    encodeFrame(encoder, 33333, {rotation: 180, flip: false}, true);
    await encoder.flush();
    assert_equals(outputs.length, 2, 'outputs');
    const configs = decoderConfigs(outputs);
    assert_equals(configs.length, 2, 'decoder configs');
    assert_equals(configs[0].rotation, 90, 'first rotation');
    assert_equals(configs[0].flip, true, 'first flip');
    assert_equals(configs[1].rotation, 180, 'second rotation');
    assert_equals(configs[1].flip, false, 'second flip');
  }, `The decoderConfig of an output describes the orientation of its frame, not the orientation set by a later configure() that ${description}`);

  promise_test(async t => {
    const {encoder, outputs} = await createEncoder(t);
    // Frames of both configurations have the same timestamp but a different orientation.
    encodeFrame(encoder, 0, {rotation: 90, flip: true});
    encoder.configure(config);
    encodeFrame(encoder, 0, {rotation: 180, flip: false}, true);
    await encoder.flush();
    assert_equals(outputs.length, 2, 'outputs');
    const configs = decoderConfigs(outputs);
    assert_equals(configs.length, 2, 'decoder configs');
    assert_equals(configs[0].rotation, 90, 'first rotation');
    assert_equals(configs[0].flip, true, 'first flip');
    assert_equals(configs[1].rotation, 180, 'second rotation');
    assert_equals(configs[1].flip, false, 'second flip');
  }, `The decoderConfig of an output does not depend on the timestamp of its frame with a configure() that ${description}`);
}
